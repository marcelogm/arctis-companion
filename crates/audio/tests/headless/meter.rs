use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use arctis_audio::samples::SAMPLE_BYTES;
use pipewire as pw;
use pw::context::ContextRc;
use pw::main_loop::MainLoopRc;
use pw::properties::properties;
use pw::spa::utils::Direction;
use pw::stream::{Stream, StreamRc};

use crate::tone::{RATE, connect};

const SETTLE_SAMPLES: u64 = RATE as u64;
const MEASURE: Duration = Duration::from_millis(1500);

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
        for sample in bytes.as_chunks::<SAMPLE_BYTES>().0 {
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
