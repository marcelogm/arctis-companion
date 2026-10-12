use gtk::glib;

pub fn on_own_context(test: impl FnOnce(&glib::MainContext)) {
    let context = glib::MainContext::new();
    context
        .with_thread_default(|| test(&context))
        .expect("the test owns its main context");
}

pub fn drain(context: &glib::MainContext) {
    while context.iteration(false) {}
}
