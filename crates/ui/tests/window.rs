use gtk::prelude::{GtkWindowExt, WidgetExt};

fn new_window() -> arctis_ui::Window {
    adw::init().expect("libadwaita initialises");
    arctis_ui::register_resources().expect("bundled resources register");
    gtk::glib::Object::new()
}

#[gtk::test]
fn window_is_titled_arctis_companion() {
    assert_eq!(new_window().title().as_deref(), Some("Arctis Companion"));
}

#[gtk::test]
fn window_keeps_the_minimum_size_of_the_design() {
    let window = new_window();
    assert_eq!(
        (window.width_request(), window.height_request()),
        (360, 294)
    );
}
