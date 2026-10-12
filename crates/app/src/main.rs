#![forbid(unsafe_code)]

use adw::prelude::*;

fn main() -> gtk::glib::ExitCode {
    arctis_audio::init();
    arctis_app::application::build().run()
}
