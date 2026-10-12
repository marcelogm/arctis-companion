use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::thread;
use std::time::Duration;

use arctis_core::eq::Curve;
use arctis_core::messages::audio::{AudioCommand, AudioEvent};
use pipewire as pw;
use pw::channel::{Receiver, Sender};
use pw::context::ContextRc;
use pw::core::CoreRc;
use pw::main_loop::{MainLoopRc, MainLoopWeak};
use pw::properties::properties;
use pw::spa::utils::result::AsyncSeq;

use crate::changes::Published;
use crate::registry::Registry;
use crate::system_engine::{Inserted, SystemEngine};

pub const RECONNECT_AFTER: Duration = Duration::from_secs(2);
pub const MANAGER_CONFIRM_AFTER: Duration = Duration::from_secs(1);
const MANAGER_CATEGORY: &str = "Manager";

pub enum Message {
    Command(AudioCommand),
    ConfirmManager,
    Retry,
    Quit,
}

#[derive(Clone)]
pub struct Link {
    pub events: async_channel::Sender<AudioEvent>,
    pub myself: Sender<Message>,
    pub announced: Rc<Cell<Option<bool>>>,
}

impl Link {
    fn announce(&self, available: bool) {
        if self.announced.replace(Some(available)) != Some(available) {
            self.send(availability(available));
        }
    }

    fn send(&self, event: AudioEvent) {
        let _ = self.events.try_send(event);
    }

    fn send_later(&self, delay: Duration, message: Message) {
        let myself = self.myself.clone();
        thread::spawn(move || {
            thread::sleep(delay);
            let _ = myself.send(message);
        });
    }
}

fn availability(available: bool) -> AudioEvent {
    if available {
        AudioEvent::Available
    } else {
        AudioEvent::Unavailable
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Lost,
    Quit,
}

pub fn run(commands: Receiver<Message>, link: &Link) {
    let mut commands = commands;
    loop {
        let (returned, outcome) = session(commands, link);
        if outcome == Outcome::Quit {
            return;
        }
        commands = returned;
    }
}

fn session(commands: Receiver<Message>, link: &Link) -> (Receiver<Message>, Outcome) {
    let Ok(main_loop) = MainLoopRc::new(None) else {
        return (commands, Outcome::Quit);
    };
    let outcome = Rc::new(Cell::new(Outcome::Lost));
    let connection = Connection::open(&main_loop, link).ok();
    if connection.is_none() {
        link.announce(false);
        link.send_later(RECONNECT_AFTER, Message::Retry);
    }
    let dispatcher = Dispatcher {
        main_loop: main_loop.downgrade(),
        worker: connection
            .as_ref()
            .map(|connection| connection.worker.clone()),
        outcome: outcome.clone(),
    };
    let attached = commands.attach(main_loop.loop_(), move |message| {
        dispatcher.dispatch(message);
    });
    main_loop.run();
    let commands = attached.deattach();
    drop(connection);
    if outcome.get() == Outcome::Lost {
        link.announce(false);
    }
    (commands, outcome.get())
}

struct Dispatcher {
    main_loop: MainLoopWeak,
    worker: Option<Rc<Worker>>,
    outcome: Rc<Cell<Outcome>>,
}

impl Dispatcher {
    fn dispatch(&self, message: Message) {
        match (message, &self.worker) {
            (Message::Quit, _) => {
                self.outcome.set(Outcome::Quit);
                self.quit();
            }
            (Message::Retry, None) => self.quit(),
            (Message::Command(command), Some(worker)) => worker.command(command),
            (Message::ConfirmManager, Some(worker)) => worker.confirm_manager(),
            _ => {}
        }
    }

    fn quit(&self) {
        if let Some(main_loop) = self.main_loop.upgrade() {
            main_loop.quit();
        }
    }
}

struct Connection {
    worker: Rc<Worker>,
    _core_listener: pw::core::Listener,
    _context: ContextRc,
}

impl Connection {
    fn open(main_loop: &MainLoopRc, link: &Link) -> Result<Self, pw::Error> {
        let context = ContextRc::new(main_loop, None)?;
        let core = context.connect_rc(Some(properties! {
            *pw::keys::MEDIA_CATEGORY => MANAGER_CATEGORY,
        }))?;
        let registry = core.get_registry_rc()?;
        let worker = Rc::new_cyclic(|weak: &Weak<Worker>| {
            let registry = Rc::new(Registry::watch(registry, publisher(weak.clone())));
            Worker {
                main_loop: main_loop.downgrade(),
                engine: SystemEngine::new(core.clone(), registry.clone()),
                registry,
                core: core.clone(),
                link: link.clone(),
                state: WorkerState::default(),
            }
        });
        let core_listener = listen_core(&core, &worker);
        worker.state.initial.set(Some(core.sync(0)?));
        Ok(Self {
            worker,
            _core_listener: core_listener,
            _context: context,
        })
    }
}

fn publisher(worker: Weak<Worker>) -> Rc<dyn Fn()> {
    Rc::new(move || {
        if let Some(worker) = worker.upgrade() {
            worker.publish();
        }
    })
}

fn listen_core(core: &CoreRc, worker: &Rc<Worker>) -> pw::core::Listener {
    let on_done = Rc::downgrade(worker);
    let on_error = Rc::downgrade(worker);
    core.add_listener_local()
        .done(move |_, seq| {
            if let Some(worker) = on_done.upgrade() {
                worker.done(seq);
            }
        })
        .error(move |id, _, _, _| {
            if let Some(worker) = on_error.upgrade().filter(|_| id == pw::sys::PW_ID_CORE) {
                worker.quit_loop();
            }
        })
        .register()
}

#[derive(Default)]
struct WorkerState {
    initial: Cell<Option<AsyncSeq>>,
    settling: Cell<Option<AsyncSeq>>,
    ready: Cell<bool>,
    published: RefCell<Published>,
}

struct Worker {
    main_loop: MainLoopWeak,
    core: CoreRc,
    registry: Rc<Registry>,
    engine: SystemEngine,
    link: Link,
    state: WorkerState,
}

impl Worker {
    fn done(&self, seq: AsyncSeq) {
        let seq = Some(seq);
        if self.state.initial.get() == seq {
            self.state.settling.set(self.core.sync(0).ok());
        } else if self.state.settling.get() == seq {
            self.become_ready();
        }
    }

    fn become_ready(&self) {
        self.state.ready.set(true);
        self.link.announce(true);
        self.publish();
    }

    fn publish(&self) {
        if !self.state.ready.get() {
            return;
        }
        let events = self
            .state
            .published
            .borrow_mut()
            .changes(&self.registry.graph());
        for event in events {
            self.link.send(event);
        }
    }

    fn command(&self, command: AudioCommand) {
        match command {
            AudioCommand::InsertEqualizer(curve) => self.insert(&curve),
            AudioCommand::Retarget(curve) => self.engine.retarget(&curve),
            AudioCommand::RemoveEqualizer => self.engine.remove(),
        }
    }

    fn insert(&self, curve: &Curve) {
        if self.engine.insert(curve) == Inserted::Created {
            self.link
                .send_later(MANAGER_CONFIRM_AFTER, Message::ConfirmManager);
        }
    }

    fn confirm_manager(&self) {
        if self.engine.manager_denied() {
            self.link.send(AudioEvent::ManagerDenied);
        }
    }

    fn quit_loop(&self) {
        if let Some(main_loop) = self.main_loop.upgrade() {
            main_loop.quit();
        }
    }
}
