#![forbid(unsafe_code)]

fn main() -> gtk::glib::ExitCode {
    arctis_audio::init();
    arctis_app::application::run()
}
