use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::intent::Intent;
use arctis_core::settings::values::Percent;
use arctis_core::view::preferences::{LOW_BATTERY_MAX, PreferencesModel};
use gtk::{gio, glib};

use crate::intents::IntentSink;
use crate::text::optional;
use crate::widget_values::nearest_step;

pub const SHELL_EXTENSIONS_URI: &str = "https://extensions.gnome.org/";

mod imp {
    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use arctis_core::view::preferences::{LOW_BATTERY_MAX, LOW_BATTERY_MIN};
    use gtk::glib;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/marcelogm/ArctisCompanion/ui/preferences_dialog.ui")]
    pub struct PreferencesDialog {
        #[template_child]
        pub run_in_background: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub low_battery: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub shell_integration: TemplateChild<adw::ActionRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesDialog {
        const NAME: &'static str = "ArctisPreferencesDialog";
        type Type = super::PreferencesDialog;
        type ParentType = adw::PreferencesDialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PreferencesDialog {
        fn constructed(&self) {
            self.parent_constructed();
            self.shell_integration
                .connect_activated(super::open_extensions);
            self.low_battery
                .set_range(f64::from(LOW_BATTERY_MIN), f64::from(LOW_BATTERY_MAX));
        }
    }

    impl WidgetImpl for PreferencesDialog {}
    impl AdwDialogImpl for PreferencesDialog {}
    impl PreferencesDialogImpl for PreferencesDialog {}
}

glib::wrapper! {
    pub struct PreferencesDialog(ObjectSubclass<imp::PreferencesDialog>)
        @extends adw::PreferencesDialog, adw::Dialog, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for PreferencesDialog {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl PreferencesDialog {
    pub fn render(&self, model: &PreferencesModel) {
        let imp = self.imp();
        imp.run_in_background.set_active(model.run_in_background);
        imp.run_in_background
            .set_subtitle(&optional(model.background_subtitle.as_ref()));
        imp.low_battery.set_value(f64::from(model.low_battery));
    }

    pub fn connect_intents(&self, sink: &IntentSink) {
        let imp = self.imp();
        let switch_sink = sink.clone();
        imp.run_in_background.connect_active_notify(move |row| {
            switch_sink.emit(Intent::SetRunInBackground(row.is_active()));
        });
        let spin_sink = sink.clone();
        imp.low_battery.connect_value_notify(move |row| {
            spin_sink.emit_some(threshold_intent(row.value()));
        });
    }
}

fn threshold_intent(value: f64) -> Option<Intent> {
    Percent::try_from(nearest_step(value, LOW_BATTERY_MAX))
        .ok()
        .map(Intent::SetLowBatteryThreshold)
}

fn open_extensions(row: &adw::ActionRow) {
    let window = row.root().and_downcast::<gtk::Window>();
    gtk::UriLauncher::new(SHELL_EXTENSIONS_URI).launch(
        window.as_ref(),
        gio::Cancellable::NONE,
        |_| {},
    );
}
