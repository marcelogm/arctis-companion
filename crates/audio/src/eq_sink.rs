use std::cell::{Cell, RefCell};
use std::io::Cursor;
use std::rc::Rc;

use arctis_core::eq::{Coefficients, Curve};
use pipewire as pw;
use pw::buffer::Buffer;
use pw::core::CoreRc;
use pw::properties::{PropertiesBox, properties};
use pw::spa;
use pw::stream::{Stream, StreamFlags, StreamListener, StreamRc};
use spa::param::ParamType;
use spa::param::audio::{AudioFormat, AudioInfoRaw, MAX_CHANNELS};
use spa::pod::serialize::PodSerializer;
use spa::pod::{Object, Pod, Value};
use spa::sys::{
    SPA_AUDIO_CHANNEL_FL, SPA_AUDIO_CHANNEL_FR, SPA_PARAM_EnumFormat, SPA_TYPE_OBJECT_Format,
};
use spa::utils::Direction;
use triple_buffer::Input;

use crate::outputs::{EQ_DESCRIPTION, EQ_LINK_GROUP, EQ_OUTPUT_NAME, EQ_SINK_NAME};
use crate::samples::{CHANNELS, FRAME_BYTES, Processor};
use crate::volume::{
    CHANNEL_VOLUMES, MUTE, SOFT_MUTE, SOFT_VOLUMES, Volume, VolumeChange, volume_change,
};

pub const INITIAL_RATE: u32 = 48_000;
const UNITY: [f32; CHANNELS] = [1.0; CHANNELS];

pub type OnVolume = Rc<dyn Fn(VolumeChange)>;

pub struct Replaced {
    pub name: String,
    pub volume: Option<Volume>,
}

pub struct EqSink {
    capture: StreamRc,
    playback: StreamRc,
    designer: Rc<RefCell<Designer>>,
    _capture_listener: StreamListener<()>,
    _playback_listener: StreamListener<Processor>,
}

impl EqSink {
    pub fn create(
        core: &CoreRc,
        curve: &Curve,
        replaced: Replaced,
        on_volume: OnVolume,
    ) -> Result<Self, pw::Error> {
        let (input, output) =
            triple_buffer::triple_buffer(&Coefficients::design(curve, INITIAL_RATE));
        let designer = Rc::new(RefCell::new(Designer {
            curve: curve.clone(),
            rate: INITIAL_RATE,
            coefficients: input,
        }));
        let capture = StreamRc::new(core.clone(), EQ_SINK_NAME, capture_props())?;
        let playback = StreamRc::new(core.clone(), EQ_OUTPUT_NAME, playback_props(&replaced.name))?;
        let reactions = Reactions {
            designer: designer.clone(),
            on_volume,
            start: replaced.volume,
            opening: Cell::new(true),
        };
        let capture_listener = listen_capture(&capture, playback.clone(), reactions)?;
        let playback_listener =
            listen_playback(&playback, capture.clone(), Processor::new(output))?;
        connect(&capture, Direction::Input, StreamFlags::empty())?;
        connect(&playback, Direction::Output, StreamFlags::TRIGGER)?;
        Ok(Self {
            capture,
            playback,
            designer,
            _capture_listener: capture_listener,
            _playback_listener: playback_listener,
        })
    }

    pub fn retarget(&self, curve: &Curve) {
        self.designer.borrow_mut().set_curve(curve);
    }
}

impl Drop for EqSink {
    fn drop(&mut self) {
        let _ = self.playback.disconnect();
        let _ = self.capture.disconnect();
    }
}

struct Designer {
    curve: Curve,
    rate: u32,
    coefficients: Input<Coefficients>,
}

impl Designer {
    fn set_curve(&mut self, curve: &Curve) {
        self.curve = curve.clone();
        self.publish();
    }

    fn set_rate(&mut self, rate: u32) {
        if rate != self.rate {
            self.rate = rate;
            self.publish();
        }
    }

    fn publish(&mut self) {
        self.coefficients
            .write(Coefficients::design(&self.curve, self.rate));
    }
}

fn capture_props() -> PropertiesBox {
    properties! {
        *pw::keys::MEDIA_TYPE => "Audio",
        *pw::keys::MEDIA_CLASS => "Audio/Sink",
        *pw::keys::NODE_NAME => EQ_SINK_NAME,
        *pw::keys::NODE_DESCRIPTION => EQ_DESCRIPTION,
        *pw::keys::NODE_LINK_GROUP => EQ_LINK_GROUP,
        *pw::keys::NODE_GROUP => EQ_LINK_GROUP,
        *pw::keys::AUDIO_CHANNELS => "2",
        "audio.position" => "[ FL FR ]",
    }
}

fn playback_props(target: &str) -> PropertiesBox {
    properties! {
        *pw::keys::MEDIA_TYPE => "Audio",
        *pw::keys::NODE_NAME => EQ_OUTPUT_NAME,
        *pw::keys::NODE_DESCRIPTION => EQ_DESCRIPTION,
        *pw::keys::NODE_LINK_GROUP => EQ_LINK_GROUP,
        *pw::keys::NODE_GROUP => EQ_LINK_GROUP,
        *pw::keys::TARGET_OBJECT => target,
        *pw::keys::NODE_DONT_RECONNECT => "true",
        *pw::keys::NODE_PASSIVE => "true",
        *pw::keys::AUDIO_CHANNELS => "2",
        "audio.position" => "[ FL FR ]",
    }
}

fn listen_capture(
    capture: &StreamRc,
    playback: StreamRc,
    reactions: Reactions,
) -> Result<StreamListener<()>, pw::Error> {
    capture
        .add_local_listener_with_user_data(())
        .param_changed(move |stream, (), id, param| {
            if let Some(param) = param {
                reactions.on_param(stream, id, param);
            }
        })
        .process(move |_, ()| {
            let _ = playback.trigger_process();
        })
        .register()
}

struct Reactions {
    designer: Rc<RefCell<Designer>>,
    on_volume: OnVolume,
    start: Option<Volume>,
    opening: Cell<bool>,
}

impl Reactions {
    fn on_param(&self, stream: &Stream, id: u32, param: &Pod) {
        if id == ParamType::Format.as_raw() {
            self.on_format(param);
        } else if id == ParamType::Props.as_raw() {
            self.on_props(stream, param);
        }
    }

    fn on_format(&self, param: &Pod) {
        if let Some(rate) = rate_of(param) {
            self.designer.borrow_mut().set_rate(rate);
        }
    }

    fn on_props(&self, stream: &Stream, param: &Pod) {
        if self.opening.replace(false) {
            start_at(stream, self.start.as_ref());
            return;
        }
        let change = volume_change(param);
        if !change.is_empty() {
            keep_unity(stream);
        }
        (self.on_volume)(change);
    }
}

fn start_at(stream: &Stream, volume: Option<&Volume>) {
    let Some(volume) = volume else {
        return;
    };
    let mute = if volume.mute { 1.0 } else { 0.0 };
    let _ = stream.set_control(CHANNEL_VOLUMES, &volume.channels);
    let _ = stream.set_control(MUTE, &[mute]);
    keep_unity(stream);
}

fn keep_unity(stream: &Stream) {
    let _ = stream.set_control(SOFT_VOLUMES, &UNITY);
    let _ = stream.set_control(SOFT_MUTE, &[0.0]);
}

fn rate_of(format: &Pod) -> Option<u32> {
    let mut info = AudioInfoRaw::new();
    info.parse(format).ok()?;
    Some(info.rate()).filter(|rate| *rate > 0)
}

fn listen_playback(
    playback: &StreamRc,
    capture: StreamRc,
    processor: Processor,
) -> Result<StreamListener<Processor>, pw::Error> {
    playback
        .add_local_listener_with_user_data(processor)
        .process(move |playback, processor| move_block(&capture, playback, processor))
        .register()
}

fn move_block(capture: &Stream, playback: &Stream, processor: &mut Processor) {
    let Some(mut input) = capture.dequeue_buffer() else {
        return;
    };
    let Some(mut output) = playback.dequeue_buffer() else {
        return;
    };
    let written = run_through(processor, &mut input, &mut output);
    finish_chunk(&mut output, written);
}

fn run_through(processor: &mut Processor, input: &mut Buffer, output: &mut Buffer) -> usize {
    let (Some(from), Some(to)) = (
        input.datas_mut().first_mut(),
        output.datas_mut().first_mut(),
    ) else {
        return 0;
    };
    let offset = usize::try_from(from.chunk().offset()).unwrap_or_default();
    let size = usize::try_from(from.chunk().size()).unwrap_or_default();
    let (Some(source), Some(target)) = (from.data(), to.data()) else {
        return 0;
    };
    let source = source.get(offset..offset + size).unwrap_or_default();
    processor.run(source, target)
}

fn finish_chunk(output: &mut Buffer, written: usize) {
    let Some(data) = output.datas_mut().first_mut() else {
        return;
    };
    let chunk = data.chunk_mut();
    *chunk.offset_mut() = 0;
    *chunk.stride_mut() = i32::try_from(FRAME_BYTES).unwrap_or_default();
    *chunk.size_mut() = u32::try_from(written).unwrap_or_default();
}

fn connect(stream: &StreamRc, direction: Direction, extra: StreamFlags) -> Result<(), pw::Error> {
    let format = format_pod();
    let mut params = [Pod::from_bytes(&format).ok_or(pw::Error::CreationFailed)?];
    let flags =
        StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS | extra;
    stream.connect(direction, None, flags, &mut params)
}

fn format_pod() -> Vec<u8> {
    let mut info = AudioInfoRaw::new();
    info.set_format(AudioFormat::F32LE);
    info.set_channels(2);
    let mut position = [0; MAX_CHANNELS];
    position[0] = SPA_AUDIO_CHANNEL_FL;
    position[1] = SPA_AUDIO_CHANNEL_FR;
    info.set_position(position);
    let format = Value::Object(Object {
        type_: SPA_TYPE_OBJECT_Format,
        id: SPA_PARAM_EnumFormat,
        properties: info.into(),
    });
    PodSerializer::serialize(Cursor::new(Vec::new()), &format)
        .map(|(cursor, _)| cursor.into_inner())
        .unwrap_or_default()
}
