use std::cell::{Cell, RefCell};
use std::rc::Rc;

use arctis_core::intent::Intent;

use crate::widget_values::while_rendering;

type Handler = Rc<dyn Fn(Intent)>;

#[derive(Clone, Default)]
pub struct IntentSink {
    handler: Rc<RefCell<Option<Handler>>>,
    rendering: Rc<Cell<bool>>,
}

impl IntentSink {
    pub fn connect(&self, handler: impl Fn(Intent) + 'static) {
        self.handler.replace(Some(Rc::new(handler)));
    }

    pub fn emit(&self, intent: Intent) {
        if self.rendering.get() {
            return;
        }
        let handler = self.handler.borrow().clone();
        if let Some(handler) = handler {
            handler(intent);
        }
    }

    pub fn emit_some(&self, intent: Option<Intent>) {
        if let Some(intent) = intent {
            self.emit(intent);
        }
    }

    pub fn rendering<T>(&self, render: impl FnOnce() -> T) -> T {
        while_rendering(&self.rendering, render)
    }
}
