use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use pipewire as pw;
use pw::context::ContextRc;
use pw::main_loop::MainLoopRc;
use pw::properties::{PropertiesBox, properties};
use pw::spa;
use pw::stream::{Stream, StreamFlags, StreamRc};
use spa::param::audio::{AudioFormat, AudioInfoRaw};
use spa::pod::serialize::PodSerializer;
use spa::pod::{Object, Pod, Value};
use spa::sys::{SPA_PARAM_EnumFormat, SPA_TYPE_OBJECT_Format};
use spa::utils::Direction;

pub const TONE_HZ: f32 = 1000.0;
const RATE: u32 = 48_000;
const CHANNELS: u32 = 2;
const AMPLITUDE: f32 = 0.25;
const SETTLE_SAMPLES: u64 = 48_000;
const MEASURE: Duration = Duration::from_millis(1500);

pub type Pairs = Vec<(&'static str, &'static str)>;

pub struct Tone {
    stop: pw::channel::Sender<()>,
    thread: Option<JoinHandle<()>>,
}

impl Tone {
    pub fn play(node: Pairs, client: Pairs) -> Self {
        let (stop, stopped) = pw::channel::channel();
        let thread = thread::spawn(move || play_until(&node, &client, stopped));
        Self {
            stop,
            thread: Some(thread),
        }
    }
}

impl Drop for Tone {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(thread) = self.thread.take() {
            thread.join().expect("tone thread");
        }
    }
}

fn props_from(pairs: &Pairs) -> PropertiesBox {
    let mut props = PropertiesBox::new();
    for (key, value) in pairs {
        props.insert(*key, *value);
    }
    props
}

fn play_until(node: &Pairs, client: &Pairs, stopped: pw::channel::Receiver<()>) {
    let main_loop = MainLoopRc::new(None).expect("tone loop");
    let context = ContextRc::new(&main_loop, None).expect("tone context");
    let core = context
        .connect_rc(Some(props_from(client)))
        .expect("tone core");
    let mut props = props_from(node);
    props.insert(*pw::keys::MEDIA_TYPE, "Audio");
    let stream = StreamRc::new(core, "arctis-test-tone", props).expect("tone stream");
    let _listener = stream
        .add_local_listener_with_user_data(0_u32)
        .process(write_sine)
        .register()
        .expect("tone listener");
    connect(&stream, Direction::Output);
    let quit = main_loop.downgrade();
    let _attached = stopped.attach(main_loop.loop_(), move |()| {
        if let Some(main_loop) = quit.upgrade() {
            main_loop.quit();
        }
    });
    main_loop.run();
}

fn write_sine(stream: &Stream, frame: &mut u32) {
    let Some(mut buffer) = stream.dequeue_buffer() else {
        return;
    };
    let Some(data) = buffer.datas_mut().first_mut() else {
        return;
    };
    let written = data.data().map_or(0, |bytes| fill(bytes, frame));
    let chunk = data.chunk_mut();
    *chunk.offset_mut() = 0;
    *chunk.stride_mut() = 8;
    *chunk.size_mut() = u32::try_from(written).expect("small buffer");
}

fn fill(bytes: &mut [u8], frame: &mut u32) -> usize {
    let (frames, _) = bytes.as_chunks_mut::<8>();
    for slot in frames.iter_mut() {
        let phase = f64::from(*frame % RATE) * f64::from(TONE_HZ) / f64::from(RATE);
        let sample = sine(phase).to_le_bytes();
        slot[..4].copy_from_slice(&sample);
        slot[4..].copy_from_slice(&sample);
        *frame = frame.wrapping_add(1);
    }
    frames.len() * 8
}

#[expect(clippy::cast_possible_truncation)]
fn sine(phase: f64) -> f32 {
    (phase * std::f64::consts::TAU).sin() as f32 * AMPLITUDE
}

fn connect(stream: &StreamRc, direction: Direction) {
    let mut info = AudioInfoRaw::new();
    info.set_format(AudioFormat::F32LE);
    info.set_rate(RATE);
    info.set_channels(CHANNELS);
    let format = Value::Object(Object {
        type_: SPA_TYPE_OBJECT_Format,
        id: SPA_PARAM_EnumFormat,
        properties: info.into(),
    });
    let bytes = PodSerializer::serialize(Cursor::new(Vec::new()), &format)
        .expect("format pod")
        .0
        .into_inner();
    let mut params = [Pod::from_bytes(&bytes).expect("pod")];
    let flags = StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS;
    stream
        .connect(direction, None, flags, &mut params)
        .expect("connect");
}

#[derive(Default)]
struct Meter {
    seen: u64,
    squares: f64,
    counted: u64,
}

impl Meter {
    fn absorb(&mut self, stream: &Stream) {
        let Some(mut buffer) = stream.dequeue_buffer() else {
            return;
        };
        let Some(data) = buffer.datas_mut().first_mut() else {
            return;
        };
        let size = usize::try_from(data.chunk().size()).expect("size");
        if let Some(bytes) = data.data() {
            self.add(&bytes[..size.min(bytes.len())]);
        }
    }

    fn add(&mut self, bytes: &[u8]) {
        for sample in bytes.as_chunks::<4>().0 {
            self.add_sample(f32::from_le_bytes(*sample));
        }
    }

    fn add_sample(&mut self, sample: f32) {
        self.seen += 1;
        if self.seen > SETTLE_SAMPLES {
            self.squares += f64::from(sample).powi(2);
            self.counted += 1;
        }
    }

    #[expect(clippy::cast_precision_loss)]
    fn db(&self) -> f64 {
        10.0 * (self.squares / self.counted as f64).log10()
    }
}

pub fn level_db(target: &str) -> f64 {
    let main_loop = MainLoopRc::new(None).expect("probe loop");
    let context = ContextRc::new(&main_loop, None).expect("probe context");
    let core = context.connect_rc(None).expect("probe core");
    let props = properties! {
        *pw::keys::MEDIA_TYPE => "Audio",
        *pw::keys::STREAM_CAPTURE_SINK => "true",
        *pw::keys::TARGET_OBJECT => target,
    };
    let stream = StreamRc::new(core, "arctis-test-probe", props).expect("probe stream");
    let meter = Rc::new(RefCell::new(Meter::default()));
    let filling = meter.clone();
    let _listener = stream
        .add_local_listener_with_user_data(())
        .process(move |stream, ()| filling.borrow_mut().absorb(stream))
        .register()
        .expect("probe listener");
    connect(&stream, Direction::Input);
    let quit = main_loop.downgrade();
    let timer = main_loop.loop_().add_timer(move |_| {
        if let Some(main_loop) = quit.upgrade() {
            main_loop.quit();
        }
    });
    timer.update_timer(Some(MEASURE), None);
    main_loop.run();
    meter.borrow().db()
}
