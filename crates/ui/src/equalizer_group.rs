use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::intent::Intent;
use arctis_core::messages::audio::SoundPath;
use arctis_core::view::equalizer_rows::{EqualizerGroupModel, NowPlayingModel, PresetComboModel};
use arctis_core::view::text::Text;
use gtk::glib;

use crate::intents::IntentSink;
use crate::text::resolve;
use crate::widget_values::{index, show_combo_row, show_if};

mod imp {
    use std::cell::RefCell;

    use adw::subclass::prelude::*;
    use arctis_core::view::equalizer_rows::EqualizerGroupModel;
    use gtk::glib;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/marcelogm/ArctisCompanion/ui/equalizer_group.ui")]
    pub struct EqualizerGroup {
        #[template_child]
        pub dongle_preset: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub bluetooth_preset: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub now_playing: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub use_route_preset: TemplateChild<gtk::Button>,
        #[template_child]
        pub automatic_presets: TemplateChild<adw::SwitchRow>,
        pub model: RefCell<Option<EqualizerGroupModel>>,
        pub warning_rows: RefCell<Vec<adw::ActionRow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EqualizerGroup {
        const NAME: &'static str = "ArctisEqualizerGroup";
        type Type = super::EqualizerGroup;
        type ParentType = adw::PreferencesGroup;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for EqualizerGroup {}
    impl WidgetImpl for EqualizerGroup {}
    impl PreferencesGroupImpl for EqualizerGroup {}
}

glib::wrapper! {
    pub struct EqualizerGroup(ObjectSubclass<imp::EqualizerGroup>)
        @extends adw::PreferencesGroup, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl EqualizerGroup {
    pub fn render(&self, model: &EqualizerGroupModel) {
        let imp = self.imp();
        let previous = imp.model.replace(Some(model.clone()));
        show_combo(&imp.dongle_preset, model.dongle_preset.as_ref());
        show_combo(&imp.bluetooth_preset, model.bluetooth_preset.as_ref());
        self.show_now_playing(model.now_playing.as_ref());
        imp.automatic_presets.set_active(model.automatic_presets);
        if previous.as_ref().map(|shown| &shown.warnings) != Some(&model.warnings) {
            self.show_warnings(&model.warnings);
        }
    }

    pub fn connect_intents(&self, sink: &IntentSink) {
        let imp = self.imp();
        self.connect_preset(&imp.dongle_preset, SoundPath::Dongle, sink);
        self.connect_preset(&imp.bluetooth_preset, SoundPath::Bluetooth, sink);
        imp.use_route_preset.connect_clicked(glib::clone!(
            #[weak(rename_to = group)]
            self,
            #[strong]
            sink,
            move |_| sink.emit_some(group.use_route_preset_intent())
        ));
        let sink = sink.clone();
        imp.automatic_presets.connect_active_notify(move |row| {
            sink.emit(Intent::SetAutomaticPresets(row.is_active()));
        });
    }

    fn connect_preset(&self, row: &adw::ComboRow, path: SoundPath, sink: &IntentSink) {
        row.connect_selected_notify(glib::clone!(
            #[weak(rename_to = group)]
            self,
            #[strong]
            sink,
            move |row| sink.emit_some(group.preset_intent(path, row.selected()))
        ));
    }

    fn preset_intent(&self, path: SoundPath, position: u32) -> Option<Intent> {
        let model = self.imp().model.borrow();
        let model = model.as_ref()?;
        let combo = match path {
            SoundPath::Dongle => model.dongle_preset.as_ref(),
            SoundPath::Bluetooth => model.bluetooth_preset.as_ref(),
        }?;
        let item = combo.items.get(index(position)?)?;
        Some(Intent::SetRoutePreset(path, item.id.clone()))
    }

    fn use_route_preset_intent(&self) -> Option<Intent> {
        let model = self.imp().model.borrow();
        let now_playing = model.as_ref()?.now_playing.as_ref()?;
        Some(now_playing.use_route_preset.clone())
    }

    fn show_now_playing(&self, model: Option<&NowPlayingModel>) {
        let imp = self.imp();
        show_if(&*imp.now_playing, model, |row, model| {
            row.set_subtitle(&resolve(&model.subtitle));
            imp.use_route_preset.set_label(&resolve(&model.button));
        });
    }

    fn show_warnings(&self, warnings: &[Text]) {
        let imp = self.imp();
        for row in imp.warning_rows.take() {
            self.remove(&row);
        }
        let rows: Vec<adw::ActionRow> = warnings.iter().map(warning_row).collect();
        for row in &rows {
            self.add(row);
        }
        imp.warning_rows.replace(rows);
    }
}

fn show_combo(row: &adw::ComboRow, model: Option<&PresetComboModel>) {
    show_if(row, model, |row, model| {
        show_combo_row(
            row,
            (&model.title, model.subtitle.as_ref()),
            model.items.iter().map(|item| &item.title),
            model.selected,
        );
    });
}

fn warning_row(text: &Text) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .name("equalizer-warning")
        .title(resolve(text))
        .use_markup(false)
        .build();
    row.add_prefix(&gtk::Image::from_icon_name("dialog-warning-symbolic"));
    row
}
