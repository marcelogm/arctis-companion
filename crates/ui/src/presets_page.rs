use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::companion::Snapshot;
use arctis_core::intent::Intent;
use arctis_core::view::preset_files::import_preset;
use arctis_core::view::presets::{PresetItem, presets_page};
use arctis_core::view::text::Text;
use gtk::glib;

use crate::preset_files::PickedText;
use crate::services::Services;
use crate::text::resolve;

pub const SHOW_PRESET_ACTION: &str = "win.show-preset";

mod imp {
    use std::cell::RefCell;
    use std::rc::Rc;

    use adw::subclass::prelude::*;
    use arctis_core::companion::Snapshot;
    use arctis_core::view::presets::PresetsPageModel;
    use gtk::glib;

    use crate::services::Services;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/marcelogm/ArctisCompanion/ui/presets_page.ui")]
    pub struct PresetsPage {
        #[template_child]
        pub import: TemplateChild<gtk::Button>,
        #[template_child]
        pub built_in: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub custom: TemplateChild<adw::PreferencesGroup>,
        pub services: RefCell<Option<Services>>,
        pub snapshot: RefCell<Option<Rc<Snapshot>>>,
        pub model: RefCell<Option<PresetsPageModel>>,
        pub built_in_rows: RefCell<Vec<adw::ActionRow>>,
        pub custom_rows: RefCell<Vec<adw::ActionRow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PresetsPage {
        const NAME: &'static str = "ArctisPresetsPage";
        type Type = super::PresetsPage;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PresetsPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().connect_import();
        }
    }

    impl WidgetImpl for PresetsPage {}
    impl NavigationPageImpl for PresetsPage {}
}

glib::wrapper! {
    pub struct PresetsPage(ObjectSubclass<imp::PresetsPage>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for PresetsPage {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl PresetsPage {
    pub fn set_services(&self, services: Services) {
        self.imp().services.replace(Some(services));
    }

    pub fn render(&self, snapshot: &Rc<Snapshot>) {
        let imp = self.imp();
        imp.snapshot.replace(Some(Rc::clone(snapshot)));
        let model = presets_page(snapshot);
        if imp.model.borrow().as_ref() == Some(&model) {
            return;
        }
        replace_rows(&imp.built_in, &imp.built_in_rows, &model.built_in);
        replace_rows(&imp.custom, &imp.custom_rows, &model.custom);
        let description = model.custom_description.as_ref().map(resolve);
        imp.custom.set_description(description.as_deref());
        imp.model.replace(Some(model));
    }

    fn connect_import(&self) {
        self.imp().import.connect_clicked(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_| page.import()
        ));
    }

    fn import(&self) {
        let Some(services) = self.imp().services.borrow().clone() else {
            return;
        };
        let done = glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |picked: PickedText| page.import_picked(&picked)
        );
        services.files.open(self.upcast_ref(), Box::new(done));
    }

    fn import_picked(&self, picked: &PickedText) {
        let imp = self.imp();
        let services = imp.services.borrow().clone();
        let snapshot = imp.snapshot.borrow().clone();
        let (Some(services), Some(snapshot)) = (services, snapshot) else {
            return;
        };
        match import_preset(&snapshot, &picked.name, &picked.text) {
            Ok(imported) => {
                services.sink.emit(Intent::SavePreset(imported.preset));
                announce_import(&services, &imported.toast, imported.clamped);
            }
            Err(problem) => services.toast(&problem),
        }
    }
}

fn announce_import(services: &Services, toast: &Text, clamped: Vec<Text>) {
    if clamped.is_empty() {
        services.toast(toast);
    } else {
        services.toast_with_details(toast, clamped);
    }
}

fn replace_rows(
    group: &adw::PreferencesGroup,
    shown: &RefCell<Vec<adw::ActionRow>>,
    items: &[PresetItem],
) {
    for row in shown.take() {
        group.remove(&row);
    }
    let rows: Vec<adw::ActionRow> = items.iter().map(preset_row).collect();
    for row in &rows {
        group.add(row);
    }
    shown.replace(rows);
}

fn preset_row(item: &PresetItem) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .name("preset-row")
        .title(resolve(&item.title))
        .use_markup(false)
        .activatable(true)
        .action_name(SHOW_PRESET_ACTION)
        .action_target(&item.id.as_str().to_variant())
        .build();
    row.add_suffix(&gtk::Image::from_icon_name("go-next-symbolic"));
    row
}
