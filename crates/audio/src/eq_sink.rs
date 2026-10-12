use std::cell::{Cell, RefCell};
use std::rc::Rc;

use arctis_core::eq::Curve;
use pipewire as pw;
use pw::buffer::Buffer;
use pw::core::CoreRc;
use pw::properties::{PropertiesBox, properties};
use pw::spa;
use pw::stream::{Stream, StreamFlags, StreamListener, StreamRc};
use spa::param::ParamType;
use spa::pod::Pod;
use spa::utils::Direction;

use crate::designer::{Designer, rate_of};
use crate::eq_names::{EQ_DESCRIPTION, EQ_LINK_GROUP, EQ_OUTPUT_NAME, EQ_SINK_NAME};
use crate::pod::format_pod;
use crate::samples::{CHANNELS, FRAME_BYTES, Processor};
use crate::volume::{
    CHANNEL_VOLUMES, MUTE, SOFT_MUTE, SOFT_VOLUMES, Volume, VolumeChange, volume_change,
};
use crate::wired::Wired;

const UNITY: [f32; CHANNELS] = [1.0; CHANNELS];

pub type OnVolume = Rc<dyn Fn(VolumeChange)>;

pub struct Replaced {
    pub name: String,
    pub volume: Option<Volume>,
}

struct Streams {
    capture: StreamRc,
    playback: StreamRc,
}

type Listeners = (StreamListener<()>, StreamListener<Processor>);

pub struct EqSink {
    designer: Rc<RefCell<Designer>>,
    wired: Wired<Streams, Listeners>,
}

impl EqSink {
    pub fn create(
        core: &CoreRc,
        curve: &Curve,
        replaced: Replaced,
        on_volume: OnVolume,
    ) -> Result<Self, pw::Error> {
        let (designer, output) = Designer::new(curve);
        let designer = Rc::new(RefCell::new(designer));
        let capture = StreamRc::new(core.clone(), EQ_SINK_NAME, capture_props())?;
        let playback = StreamRc::new(core.clone(), EQ_OUTPUT_NAME, playback_props(&replaced.name))?;
        let reactions = Reactions {
            designer: designer.clone(),
            on_volume,
            initial_volume: replaced.volume,
            awaiting_first_props: Cell::new(true),
        };
        let capture_listener = listen_capture(&capture, playback.clone(), reactions)?;
        let playback_listener =
            listen_playback(&playback, capture.clone(), Processor::new(output))?;
        connect(&capture, Direction::Input, StreamFlags::empty())?;
        connect(&playback, Direction::Output, StreamFlags::TRIGGER)?;
        Ok(Self {
            designer,
            wired: Wired::new(
                Streams { capture, playback },
                (capture_listener, playback_listener),
            ),
        })
    }

    pub fn retarget(&self, curve: &Curve) {
        self.designer.borrow_mut().set_curve(curve);
    }
}

impl Drop for EqSink {
    fn drop(&mut self) {
        let streams = self.wired.streams();
        let _ = streams.playback.disconnect();
        let _ = streams.capture.disconnect();
    }
}

fn common_eq_props(name: &str) -> PropertiesBox {
    properties! {
        *pw::keys::MEDIA_TYPE => "Audio",
        *pw::keys::NODE_NAME => name,
        *pw::keys::NODE_DESCRIPTION => EQ_DESCRIPTION,
        *pw::keys::NODE_LINK_GROUP => EQ_LINK_GROUP,
        *pw::keys::NODE_GROUP => EQ_LINK_GROUP,
        *pw::keys::AUDIO_CHANNELS => "2",
        "audio.position" => "[ FL FR ]",
    }
}

fn capture_props() -> PropertiesBox {
    let mut props = common_eq_props(EQ_SINK_NAME);
    props.insert(*pw::keys::MEDIA_CLASS, "Audio/Sink");
    props
}

fn playback_props(target: &str) -> PropertiesBox {
    let mut props = common_eq_props(EQ_OUTPUT_NAME);
    props.insert(*pw::keys::TARGET_OBJECT, target);
    props.insert(*pw::keys::NODE_DONT_RECONNECT, "true");
    props.insert(*pw::keys::NODE_PASSIVE, "true");
    props
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
    initial_volume: Option<Volume>,
    awaiting_first_props: Cell<bool>,
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
        if self.awaiting_first_props.replace(false) {
            apply_initial_volume(stream, self.initial_volume.as_ref());
            return;
        }
        let change = volume_change(param);
        if !change.is_empty() {
            keep_unity(stream);
        }
        (self.on_volume)(change);
    }
}

fn apply_initial_volume(stream: &Stream, volume: Option<&Volume>) {
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
    let format = format_pod(None);
    let mut params = [Pod::from_bytes(&format).ok_or(pw::Error::CreationFailed)?];
    let flags =
        StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS | extra;
    stream.connect(direction, None, flags, &mut params)
}
