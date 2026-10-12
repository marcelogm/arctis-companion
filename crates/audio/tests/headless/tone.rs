use std::thread::{self, JoinHandle};

use arctis_audio::pod::format_pod;
use arctis_audio::samples::{FRAME_BYTES, SAMPLE_BYTES};
use pipewire as pw;
use pw::context::ContextRc;
use pw::main_loop::MainLoopRc;
use pw::properties::PropertiesBox;
use pw::spa;
use pw::stream::{Stream, StreamFlags, StreamRc};
use spa::pod::Pod;
use spa::utils::Direction;

pub const TONE_HZ: f32 = 1000.0;
pub const RATE: u32 = 48_000;
const AMPLITUDE: f32 = 0.25;

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
    *chunk.stride_mut() = i32::try_from(FRAME_BYTES).expect("small frame");
    *chunk.size_mut() = u32::try_from(written).expect("small buffer");
}

fn fill(bytes: &mut [u8], frame: &mut u32) -> usize {
    let (frames, _) = bytes.as_chunks_mut::<FRAME_BYTES>();
    for slot in frames.iter_mut() {
        let phase = f64::from(*frame % RATE) * f64::from(TONE_HZ) / f64::from(RATE);
        let sample = sine(phase).to_le_bytes();
        for channel in slot.as_chunks_mut::<SAMPLE_BYTES>().0 {
            channel.copy_from_slice(&sample);
        }
        *frame = frame.wrapping_add(1);
    }
    frames.len() * FRAME_BYTES
}

#[expect(clippy::cast_possible_truncation)]
fn sine(phase: f64) -> f32 {
    (phase * std::f64::consts::TAU).sin() as f32 * AMPLITUDE
}

pub fn connect(stream: &StreamRc, direction: Direction) {
    let bytes = format_pod(Some(RATE));
    let mut params = [Pod::from_bytes(&bytes).expect("pod")];
    let flags = StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS;
    stream
        .connect(direction, None, flags, &mut params)
        .expect("connect");
}
