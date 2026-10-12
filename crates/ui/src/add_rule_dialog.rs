use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::eq::preset::PresetId;
use arctis_core::intent::Intent;
use arctis_core::view::app_presets::{AddRuleModel, SeenRowModel};
use arctis_core::view::presets::PresetItem;
use gtk::glib;

use crate::app_icons::display_icon_name;
use crate::intents::IntentSink;
use crate::text::resolve;
use crate::widget_values::{index, show_choices};

mod imp {
    use std::cell::RefCell;

    use adw::subclass::prelude::*;
    use arctis_core::eq::preset::PresetId;
    use arctis_core::view::app_presets::SeenRowModel;
    use gtk::glib;

    use crate::intents::IntentSink;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/marcelogm/ArctisCompanion/ui/add_rule_dialog.ui")]
    pub struct AddRuleDialog {
        #[template_child]
        pub preset: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub seen_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub other_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub other_app: TemplateChild<adw::EntryRow>,
        pub seen_rows: RefCell<Vec<adw::ActionRow>>,
        pub shown_seen: RefCell<Vec<SeenRowModel>>,
        pub choices: RefCell<Vec<PresetId>>,
        pub sink: RefCell<Option<IntentSink>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AddRuleDialog {
        const NAME: &'static str = "ArctisAddRuleDialog";
        type Type = super::AddRuleDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AddRuleDialog {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().connect_other_app();
        }
    }

    impl WidgetImpl for AddRuleDialog {}
    impl AdwDialogImpl for AddRuleDialog {}
}

glib::wrapper! {
    pub struct AddRuleDialog(ObjectSubclass<imp::AddRuleDialog>)
        @extends adw::Dialog, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for AddRuleDialog {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl AddRuleDialog {
    pub fn connect_intents(&self, sink: &IntentSink) {
        self.imp().sink.replace(Some(sink.clone()));
    }

    pub fn render(&self, model: &AddRuleModel, presets: &[PresetItem]) {
        let imp = self.imp();
        self.show_presets(presets);
        imp.other_app.set_title(&resolve(&model.other_title));
        imp.other_group
            .set_description(Some(&resolve(&model.other_hint)));
        imp.seen_group.set_visible(!model.seen.is_empty());
        if *imp.shown_seen.borrow() != model.seen {
            self.show_seen(&model.seen);
        }
    }

    fn connect_other_app(&self) {
        self.imp().other_app.connect_apply(glib::clone!(
            #[weak(rename_to = dialog)]
            self,
            move |entry| dialog.add_rule(&entry.text())
        ));
    }

    pub fn open(&self, parent: &impl IsA<gtk::Widget>) {
        let imp = self.imp();
        self.present(Some(parent));
        if !imp.seen_group.get_visible() {
            self.set_focus(Some(&*imp.other_app));
        }
    }

    fn show_presets(&self, presets: &[PresetItem]) {
        let keep = self
            .chosen_preset()
            .and_then(|chosen| presets.iter().position(|item| item.id == chosen))
            .unwrap_or_default();
        let titles: Vec<String> = presets.iter().map(|item| resolve(&item.title)).collect();
        let imp = self.imp();
        imp.choices
            .replace(presets.iter().map(|item| item.id.clone()).collect());
        show_choices(&imp.preset, &titles, keep);
    }

    fn show_seen(&self, seen: &[SeenRowModel]) {
        let imp = self.imp();
        for row in imp.seen_rows.take() {
            imp.seen_group.remove(&row);
        }
        let rows: Vec<adw::ActionRow> = seen.iter().map(|app| self.seen_row(app)).collect();
        for row in &rows {
            imp.seen_group.add(row);
        }
        imp.seen_rows.replace(rows);
        imp.shown_seen.replace(seen.to_vec());
    }

    fn seen_row(&self, app: &SeenRowModel) -> adw::ActionRow {
        let row = adw::ActionRow::builder()
            .name("seen-app")
            .title(resolve(&app.title))
            .subtitle(resolve(&app.subtitle))
            .use_markup(false)
            .activatable(true)
            .build();
        row.add_prefix(&app_icon(self, &app.identifier));
        let identifier = app.identifier.clone();
        row.connect_activated(glib::clone!(
            #[weak(rename_to = dialog)]
            self,
            move |_| dialog.add_rule(&identifier)
        ));
        row
    }

    fn add_rule(&self, identifier: &str) {
        if let Some(intent) = rule_intent(identifier, self.chosen_preset()) {
            self.emit(intent);
            self.imp().other_app.set_text("");
            self.close();
        }
    }

    fn chosen_preset(&self) -> Option<PresetId> {
        let imp = self.imp();
        imp.choices
            .borrow()
            .get(index(imp.preset.selected())?)
            .cloned()
    }

    fn emit(&self, intent: Intent) {
        let sink = self.imp().sink.borrow().clone();
        if let Some(sink) = sink {
            sink.emit(intent);
        }
    }
}

fn rule_intent(identifier: &str, preset: Option<PresetId>) -> Option<Intent> {
    let preset = preset?;
    let identifier = identifier.trim();
    (!identifier.is_empty()).then(|| Intent::SetAppRule {
        identifier: identifier.to_owned(),
        preset,
    })
}

fn app_icon(dialog: &AddRuleDialog, identifier: &str) -> gtk::Image {
    gtk::Image::builder()
        .icon_name(display_icon_name(dialog, identifier))
        .icon_size(gtk::IconSize::Large)
        .build()
}
