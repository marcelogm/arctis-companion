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

use crate::changes::{Published, Settling, Ticket};
use crate::eq_names::EQ_SINK_NAME;
use crate::insertion::RemoveStep;
use crate::registry::Registry;
use crate::system_engine::{Inserted, SystemEngine};

const RECONNECT_AFTER: Duration = Duration::from_secs(2);
const MANAGER_CONFIRM_AFTER: Duration = Duration::from_secs(1);
const SETTLE_WITHIN: Duration = Duration::from_secs(1);
const SHUTDOWN_GRACE: Duration = Duration::from_secs(1);
const MANAGER_CATEGORY: &str = "Manager";

pub enum Message {
    Command(AudioCommand),
    ConfirmManager,
    Settled(Ticket),
    Retry,
    Quit,
}

#[derive(Clone)]
pub struct Outbox {
    events: async_channel::Sender<AudioEvent>,
    to_itself: Sender<Message>,
    announced: Rc<Cell<Option<bool>>>,
}

impl Outbox {
    pub fn new(events: async_channel::Sender<AudioEvent>, to_itself: Sender<Message>) -> Self {
        Self {
            events,
            to_itself,
            announced: Rc::default(),
        }
    }

    fn announce(&self, available: bool) {
        if self.announced.replace(Some(available)) != Some(available) {
            self.send(availability(available));
        }
    }

    fn send(&self, event: AudioEvent) {
        let _ = self.events.try_send(event);
    }

    fn send_later(&self, delay: Duration, message: Message) {
        let to_itself = self.to_itself.clone();
        thread::spawn(move || {
            thread::sleep(delay);
            let _ = to_itself.send(message);
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

pub fn run(commands: Receiver<Message>, outbox: &Outbox) {
    let mut commands = commands;
    loop {
        let (returned, outcome) = session(commands, outbox);
        if outcome == Outcome::Quit {
            return;
        }
        commands = returned;
    }
}

fn session(commands: Receiver<Message>, outbox: &Outbox) -> (Receiver<Message>, Outcome) {
    let Ok(main_loop) = MainLoopRc::new(None) else {
        return (commands, Outcome::Quit);
    };
    let outcome = Rc::new(Cell::new(Outcome::Lost));
    let connection = Connection::open(&main_loop, outbox).ok();
    if connection.is_none() {
        outbox.announce(false);
        outbox.send_later(RECONNECT_AFTER, Message::Retry);
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
        outbox.announce(false);
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
            (Message::Quit, Some(worker)) => {
                self.outcome.set(Outcome::Quit);
                worker.shut_down();
            }
            (Message::Quit, None) => {
                self.outcome.set(Outcome::Quit);
                self.quit();
            }
            (Message::Retry, None) => self.quit(),
            (Message::Command(command), Some(worker)) => worker.command(command),
            (Message::ConfirmManager, Some(worker)) => worker.confirm_manager(),
            (Message::Settled(ticket), Some(worker)) => worker.stop_settling(ticket),
            (Message::Command(_) | Message::ConfirmManager | Message::Settled(_), None)
            | (Message::Retry, Some(_)) => {}
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
    fn open(main_loop: &MainLoopRc, outbox: &Outbox) -> Result<Self, pw::Error> {
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
                outbox: outbox.clone(),
                state: WorkerState::default(),
            }
        });
        let core_listener = listen_core(&core, &worker);
        worker.state.globals_listed.set(Some(core.sync(0)?));
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
            worker.on_graph_changed();
        }
    })
}

fn listen_core(core: &CoreRc, worker: &Rc<Worker>) -> pw::core::Listener {
    let on_done = Rc::downgrade(worker);
    let on_error = Rc::downgrade(worker);
    core.add_listener_local()
        .done(move |_, seq| {
            if let Some(worker) = on_done.upgrade() {
                worker.on_roundtrip_done(seq);
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
    globals_listed: Cell<Option<AsyncSeq>>,
    infos_received: Cell<Option<AsyncSeq>>,
    restoring_default: Cell<bool>,
    ready: Cell<bool>,
    published: RefCell<Published>,
}

struct Worker {
    main_loop: MainLoopWeak,
    core: CoreRc,
    registry: Rc<Registry>,
    engine: SystemEngine,
    outbox: Outbox,
    state: WorkerState,
}

impl Worker {
    fn on_roundtrip_done(&self, seq: AsyncSeq) {
        let seq = Some(seq);
        if self.state.globals_listed.get() == seq {
            self.state.infos_received.set(self.core.sync(0).ok());
        } else if self.state.infos_received.get() == seq {
            self.become_ready();
        }
    }

    fn become_ready(&self) {
        self.state.ready.set(true);
        self.outbox.announce(true);
        self.publish();
    }

    fn on_graph_changed(&self) {
        self.quit_once_restored();
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
            self.outbox.send(event);
        }
    }

    fn command(&self, command: AudioCommand) {
        match command {
            AudioCommand::InsertEqualizer(curve) => self.insert(&curve),
            AudioCommand::Retarget(curve) => self.engine.retarget(&curve),
            AudioCommand::RemoveEqualizer => self.remove(),
        }
    }

    fn insert(&self, curve: &Curve) {
        if self.engine.insert(curve) == Inserted::Created {
            self.settle(Settling::Insertion);
            self.outbox
                .send_later(MANAGER_CONFIRM_AFTER, Message::ConfirmManager);
        }
    }

    fn remove(&self) {
        if self.engine.remove() != RemoveStep::NothingInserted {
            self.settle(Settling::Removal);
        }
    }

    fn settle(&self, settling: Settling) {
        let ticket = self.state.published.borrow_mut().settle(settling);
        self.outbox
            .send_later(SETTLE_WITHIN, Message::Settled(ticket));
    }

    fn stop_settling(&self, ticket: Ticket) {
        self.state.published.borrow_mut().stop_settling(ticket);
        self.publish();
    }

    fn confirm_manager(&self) {
        if self.engine.manager_denied() {
            self.outbox.send(AudioEvent::ManagerDenied);
        }
    }

    fn shut_down(&self) {
        if self.state.restoring_default.replace(true) {
            self.quit_loop();
            return;
        }
        self.engine.remove();
        self.outbox.send_later(SHUTDOWN_GRACE, Message::Quit);
        self.quit_once_restored();
    }

    fn quit_once_restored(&self) {
        let restored = self.registry.graph().defaults().configured() != Some(EQ_SINK_NAME);
        if self.state.restoring_default.get() && restored {
            self.quit_loop();
        }
    }

    fn quit_loop(&self) {
        if let Some(main_loop) = self.main_loop.upgrade() {
            main_loop.quit();
        }
    }
}
