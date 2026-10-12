use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::view::app_presets::{
    AddRuleModel, AppPresetsContent, AppPresetsModel, RuleRowModel,
};
use arctis_core::view::presets::PresetItem;
use arctis_core::view::status_page::StatusPageModel;
use arctis_core::view::text::Text;
use gtk::glib;

use crate::add_rule_dialog::AddRuleDialog;
use crate::app_rule_row::AppRuleRow;
use crate::intents::IntentSink;
use crate::text::resolve;
use crate::widget_values::show_if;

const ADD_RULE_ACTION: &str = "app-presets.add-rule";

pub struct AddRuleInputs {
    model: AddRuleModel,
    presets: Vec<PresetItem>,
}

mod imp {
    use std::cell::{OnceCell, RefCell};

    use adw::subclass::prelude::*;
    use gtk::glib;
    use gtk::glib::prelude::StaticTypeExt;

    use super::AddRuleInputs;
    use crate::add_rule_dialog::AddRuleDialog;
    use crate::app_rule_row::AppRuleRow;
    use crate::intents::IntentSink;
    use crate::status_pages::StatusView;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/marcelogm/ArctisCompanion/ui/app_presets_page.ui")]
    pub struct AppPresetsPage {
        #[template_child]
        pub add_rule: TemplateChild<gtk::Button>,
        #[template_child]
        pub content_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub empty_view: TemplateChild<StatusView>,
        #[template_child]
        pub rules_page: TemplateChild<adw::PreferencesPage>,
        #[template_child]
        pub automatic_off_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub automatic_off: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub rules_group: TemplateChild<adw::PreferencesGroup>,
        pub rows: RefCell<Vec<AppRuleRow>>,
        pub sink: RefCell<IntentSink>,
        pub add_rule_inputs: RefCell<Option<AddRuleInputs>>,
        pub dialog: OnceCell<AddRuleDialog>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AppPresetsPage {
        const NAME: &'static str = "ArctisAppPresetsPage";
        type Type = super::AppPresetsPage;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            StatusView::ensure_type();
            klass.bind_template();
            klass.install_action(super::ADD_RULE_ACTION, None, |page, _, _| {
                page.add_rule_dialog().open(page);
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AppPresetsPage {}
    impl WidgetImpl for AppPresetsPage {}
    impl NavigationPageImpl for AppPresetsPage {}
}

glib::wrapper! {
    pub struct AppPresetsPage(ObjectSubclass<imp::AppPresetsPage>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl AppPresetsPage {
    pub fn connect_intents(&self, sink: &IntentSink) {
        self.imp().sink.replace(sink.clone());
    }

    pub fn render(&self, model: &AppPresetsModel, add_rule: &AddRuleModel) {
        let imp = self.imp();
        imp.rules_page.set_description(&resolve(&model.description));
        self.show_automatic_off(model.automatic_off.as_ref());
        match &model.content {
            AppPresetsContent::Empty(empty) => self.show_empty(empty),
            AppPresetsContent::Rules(rows) => self.show_rules(rows, &model.presets),
        }
        self.keep_add_rule_inputs(add_rule, &model.presets);
    }

    #[must_use]
    pub fn add_rule_dialog(&self) -> AddRuleDialog {
        let imp = self.imp();
        imp.dialog
            .get_or_init(|| {
                let dialog = AddRuleDialog::default();
                dialog.connect_intents(&imp.sink.borrow());
                self.render_dialog(&dialog);
                dialog
            })
            .clone()
    }

    fn keep_add_rule_inputs(&self, model: &AddRuleModel, presets: &[PresetItem]) {
        let imp = self.imp();
        imp.add_rule_inputs.replace(Some(AddRuleInputs {
            model: model.clone(),
            presets: presets.to_vec(),
        }));
        if let Some(dialog) = imp.dialog.get() {
            self.render_dialog(dialog);
        }
    }

    fn render_dialog(&self, dialog: &AddRuleDialog) {
        let imp = self.imp();
        if let Some(inputs) = imp.add_rule_inputs.borrow().as_ref() {
            imp.sink
                .borrow()
                .rendering(|| dialog.render(&inputs.model, &inputs.presets));
        }
    }

    fn show_automatic_off(&self, warning: Option<&Text>) {
        let imp = self.imp();
        show_if(&*imp.automatic_off_group, warning, |_, text| {
            imp.automatic_off.set_title(&resolve(text));
        });
    }

    fn show_empty(&self, empty: &StatusPageModel) {
        let imp = self.imp();
        imp.empty_view.render(empty, Some(ADD_RULE_ACTION));
        self.sync_rows(&[], &[]);
        imp.add_rule.set_visible(false);
        imp.content_stack.set_visible_child_name("empty");
    }

    fn show_rules(&self, rows: &[RuleRowModel], presets: &[PresetItem]) {
        let imp = self.imp();
        self.sync_rows(rows, presets);
        imp.add_rule.set_visible(true);
        imp.content_stack.set_visible_child_name("rules");
    }

    fn sync_rows(&self, models: &[RuleRowModel], presets: &[PresetItem]) {
        if !self.rows_match(models) {
            self.rebuild_rows(models.len());
        }
        for (row, model) in self.imp().rows.borrow().iter().zip(models) {
            row.render(model, presets);
        }
    }

    fn rows_match(&self, models: &[RuleRowModel]) -> bool {
        let rows = self.imp().rows.borrow();
        rows.len() == models.len()
            && rows
                .iter()
                .zip(models)
                .all(|(row, model)| row.identifier() == model.identifier)
    }

    fn rebuild_rows(&self, count: usize) {
        let imp = self.imp();
        for row in imp.rows.take() {
            imp.rules_group.remove(&row);
        }
        let sink = imp.sink.borrow().clone();
        let rows: Vec<AppRuleRow> = (0..count).map(|_| AppRuleRow::new(&sink)).collect();
        for row in &rows {
            imp.rules_group.add(row);
        }
        imp.rows.replace(rows);
    }
}
