use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::eq::preset::PresetId;
use arctis_core::intent::Intent;
use arctis_core::view::app_presets::RuleRowModel;
use arctis_core::view::presets::PresetItem;
use gtk::glib;

use crate::app_icons::display_icon_name;
use crate::intents::IntentSink;
use crate::text::resolve;
use crate::widget_values::{chosen, show_choices};

mod imp {
    use std::cell::RefCell;

    use adw::subclass::prelude::*;
    use arctis_core::eq::preset::PresetId;
    use gtk::glib;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/marcelogm/ArctisCompanion/ui/app_rule_row.ui")]
    pub struct AppRuleRow {
        #[template_child]
        pub icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub remove: TemplateChild<gtk::Button>,
        pub identifier: RefCell<String>,
        pub choices: RefCell<Vec<Option<PresetId>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AppRuleRow {
        const NAME: &'static str = "ArctisAppRuleRow";
        type Type = super::AppRuleRow;
        type ParentType = adw::ComboRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AppRuleRow {}
    impl WidgetImpl for AppRuleRow {}
    impl ListBoxRowImpl for AppRuleRow {}
    impl PreferencesRowImpl for AppRuleRow {}
    impl ActionRowImpl for AppRuleRow {}
    impl ComboRowImpl for AppRuleRow {}
}

glib::wrapper! {
    pub struct AppRuleRow(ObjectSubclass<imp::AppRuleRow>)
        @extends adw::ComboRow, adw::ActionRow, adw::PreferencesRow, gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl AppRuleRow {
    #[must_use]
    pub fn new(sink: &IntentSink) -> Self {
        let row: Self = glib::Object::new();
        row.connect_intents(sink);
        row
    }

    #[must_use]
    pub fn identifier(&self) -> String {
        self.imp().identifier.borrow().clone()
    }

    pub fn render(&self, model: &RuleRowModel, presets: &[PresetItem]) {
        let imp = self.imp();
        imp.identifier.replace(model.identifier.clone());
        self.set_title(&resolve(&model.title));
        self.set_subtitle(&resolve(&model.subtitle));
        imp.icon
            .set_icon_name(Some(&display_icon_name(self, &model.identifier)));
        self.show_presets(model, presets);
    }

    fn show_presets(&self, model: &RuleRowModel, presets: &[PresetItem]) {
        let choices = rule_choices(model, presets);
        let selected = choices
            .iter()
            .position(|(id, _)| *id == model.preset)
            .unwrap_or_default();
        let titles: Vec<String> = choices.iter().map(|(_, title)| title.clone()).collect();
        self.imp()
            .choices
            .replace(choices.into_iter().map(|(id, _)| id).collect());
        show_choices(self.upcast_ref(), &titles, selected);
    }

    fn connect_intents(&self, sink: &IntentSink) {
        self.connect_selected_notify(glib::clone!(
            #[strong]
            sink,
            move |row| sink.emit_some(row.chosen_intent())
        ));
        self.imp().remove.connect_clicked(glib::clone!(
            #[weak(rename_to = row)]
            self,
            #[strong]
            sink,
            move |_| sink.emit(Intent::RemoveAppRule(row.identifier()))
        ));
    }

    fn chosen_preset(&self) -> Option<PresetId> {
        chosen(&self.imp().choices.borrow(), self.selected())?
    }

    fn chosen_intent(&self) -> Option<Intent> {
        let preset = self.chosen_preset()?;
        Some(Intent::SetAppRule {
            identifier: self.identifier(),
            preset,
        })
    }
}

fn rule_choices(model: &RuleRowModel, presets: &[PresetItem]) -> Vec<(Option<PresetId>, String)> {
    let library = presets
        .iter()
        .map(|item| (Some(item.id.clone()), resolve(&item.title)));
    missing_choice(model).into_iter().chain(library).collect()
}

fn missing_choice(model: &RuleRowModel) -> Option<(Option<PresetId>, String)> {
    match model.preset {
        None => Some((None, resolve(&model.subtitle))),
        Some(_) => None,
    }
}
