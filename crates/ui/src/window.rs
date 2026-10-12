use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::battery::sample::UnixSeconds;
use arctis_core::companion::Snapshot;
use arctis_core::eq::preset::PresetId;
use arctis_core::intent::Intent;
use arctis_core::view::app_presets::{add_rule_dialog, app_presets};
use arctis_core::view::headset_page::{
    BannerModel, HeadsetModel, HeadsetPageModel, StatusKind, headset_page,
};
use arctis_core::view::preferences::{Background, preferences};
use arctis_core::view::status_page::StatusPageModel;
use gtk::{gio, glib};

use crate::eq_editor_page::EDITOR_TAG;
use crate::preferences_dialog::PreferencesDialog;
use crate::preset_files::{PresetFiles, portal_files};
use crate::presets_page::SHOW_PRESET_ACTION;
use crate::services::Services;
use crate::text::resolve;

const PERMISSION_ACTION: &str = "win.permission-instructions";
const MICROS_PER_SECOND: i64 = 1_000_000;
const BATTERY_PAGE: &str = "battery";
const BATTERY_REFRESH_SECONDS: u32 = 60;

mod imp {
    use std::cell::{Cell, OnceCell, RefCell};
    use std::rc::Rc;

    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use arctis_core::companion::Snapshot;
    use arctis_core::view::preferences::Background;
    use gtk::{gdk, glib};

    use crate::app_dialogs::{about_dialog, shortcuts_dialog};
    use crate::app_presets_page::AppPresetsPage;
    use crate::battery_page::BatteryPage;
    use crate::eq_editor_page::EqEditorPage;
    use crate::headset_page::HeadsetPage;
    use crate::intents::IntentSink;
    use crate::permission_dialog::PermissionDialog;
    use crate::preferences_dialog::PreferencesDialog;
    use crate::preset_files::PresetFiles;
    use crate::presets_page::PresetsPage;
    use crate::status_pages::StatusView;
    use crate::toasts::ToastFeed;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/marcelogm/ArctisCompanion/ui/window.ui")]
    pub struct Window {
        #[template_child]
        pub toasts: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub navigation: TemplateChild<adw::NavigationView>,
        #[template_child]
        pub banner: TemplateChild<adw::Banner>,
        #[template_child]
        pub main_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub status_view: TemplateChild<StatusView>,
        #[template_child]
        pub headset_page: TemplateChild<HeadsetPage>,
        #[template_child]
        pub app_presets_page: TemplateChild<AppPresetsPage>,
        #[template_child]
        pub battery_page: TemplateChild<BatteryPage>,
        #[template_child]
        pub presets_page: TemplateChild<PresetsPage>,
        #[template_child]
        pub eq_editor_page: TemplateChild<EqEditorPage>,
        pub files: RefCell<Option<Rc<dyn PresetFiles>>>,
        pub sink: IntentSink,
        pub toast_feed: ToastFeed,
        pub snapshot: RefCell<Option<Rc<Snapshot>>>,
        pub battery_clock: RefCell<Option<glib::SourceId>>,
        pub background: Cell<Background>,
        pub preferences: OnceCell<PreferencesDialog>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "ArctisWindow";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            StatusView::ensure_type();
            HeadsetPage::ensure_type();
            AppPresetsPage::ensure_type();
            BatteryPage::ensure_type();
            PresetsPage::ensure_type();
            EqEditorPage::ensure_type();
            klass.bind_template();
            klass.install_action("win.preferences", None, |window, _, _| {
                window.preferences_dialog().present(Some(window));
            });
            klass.install_action("win.about", None, |window, _, _| {
                about_dialog().present(Some(window));
            });
            klass.install_action("win.shortcuts", None, |window, _, _| {
                shortcuts_dialog().present(Some(window));
            });
            klass.install_action(super::PERMISSION_ACTION, None, |window, _, _| {
                PermissionDialog::default().present(Some(window));
            });
            klass.install_action(
                "win.show-page",
                Some(glib::VariantTy::STRING),
                |window, _, target| window.show_page(target.and_then(glib::Variant::str)),
            );
            klass.install_action(
                super::SHOW_PRESET_ACTION,
                Some(glib::VariantTy::STRING),
                |window, _, target| window.show_preset(target.and_then(glib::Variant::str)),
            );
            klass.add_binding_action(
                gdk::Key::comma,
                gdk::ModifierType::CONTROL_MASK,
                "win.preferences",
            );
            klass.add_binding_action(
                gdk::Key::question,
                gdk::ModifierType::CONTROL_MASK,
                "win.shortcuts",
            );
            klass.add_binding_action(gdk::Key::w, gdk::ModifierType::CONTROL_MASK, "window.close");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();
            self.headset_page.connect_intents(&self.sink);
            self.app_presets_page.connect_intents(&self.sink);
            self.obj().keep_battery_current_while_shown();
            self.obj().connect_preset_pages();
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    #[must_use]
    pub fn new(application: &adw::Application) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }

    pub fn on_intent(&self, handler: impl Fn(Intent) + 'static) {
        self.imp().sink.connect(handler);
    }

    pub fn render(&self, snapshot: &Rc<Snapshot>) {
        let imp = self.imp();
        imp.snapshot.replace(Some(Rc::clone(snapshot)));
        imp.sink.rendering(|| {
            self.render_main(&headset_page(snapshot));
            self.render_app_presets(snapshot);
            self.render_preferences();
            self.render_preset_pages(snapshot);
        });
        if self.battery_page_is_visible() {
            self.refresh_battery();
        }
    }

    fn battery_page_is_visible(&self) -> bool {
        let visible = self.imp().navigation.visible_page();
        visible.and_then(|page| page.tag()).as_deref() == Some(BATTERY_PAGE)
    }

    #[must_use]
    pub fn is_refreshing_battery(&self) -> bool {
        self.imp().battery_clock.borrow().is_some()
    }

    fn keep_battery_current_while_shown(&self) {
        let page = &self.imp().battery_page;
        let window = self.downgrade();
        page.connect_showing(move |_| {
            window.upgrade().inspect(Self::start_battery_clock);
        });
        let window = self.downgrade();
        page.connect_hiding(move |_| {
            window.upgrade().inspect(Self::stop_battery_clock);
        });
    }

    fn start_battery_clock(&self) {
        self.stop_battery_clock();
        self.refresh_battery();
        let window = self.downgrade();
        let clock = glib::timeout_add_seconds_local(BATTERY_REFRESH_SECONDS, move || {
            refresh_battery_of(&window)
        });
        self.imp().battery_clock.replace(Some(clock));
    }

    fn stop_battery_clock(&self) {
        if let Some(clock) = self.imp().battery_clock.take() {
            clock.remove();
        }
    }

    fn refresh_battery(&self) {
        let imp = self.imp();
        let snapshot = imp.snapshot.borrow().clone();
        if let Some(snapshot) = snapshot {
            imp.battery_page.display_history(snapshot, current_time());
        }
    }

    pub fn set_background(&self, background: Background) {
        let imp = self.imp();
        imp.background.set(background);
        imp.sink.rendering(|| self.render_preferences());
    }

    #[must_use]
    pub fn preferences_dialog(&self) -> PreferencesDialog {
        let imp = self.imp();
        imp.preferences
            .get_or_init(|| {
                let dialog = PreferencesDialog::default();
                dialog.connect_intents(&imp.sink);
                dialog
            })
            .clone()
    }

    fn render_preferences(&self) {
        let imp = self.imp();
        let snapshot = imp.snapshot.borrow().clone();
        if let Some(snapshot) = snapshot {
            let model = preferences(&snapshot, imp.background.get());
            self.preferences_dialog().render(&model);
        }
    }

    pub fn set_preset_files(&self, files: Rc<dyn PresetFiles>) {
        self.imp().files.replace(Some(files));
        self.connect_preset_pages();
    }

    fn connect_preset_pages(&self) {
        let imp = self.imp();
        let services = Services {
            sink: imp.sink.clone(),
            toasts: imp.toasts.get(),
            files: imp.files.borrow().clone().unwrap_or_else(portal_files),
        };
        imp.presets_page.set_services(services.clone());
        imp.eq_editor_page.set_services(services);
    }

    fn render_preset_pages(&self, snapshot: &Rc<Snapshot>) {
        let imp = self.imp();
        imp.presets_page.render(snapshot);
        imp.eq_editor_page.render(snapshot);
    }

    fn show_preset(&self, id: Option<&str>) {
        let imp = self.imp();
        let snapshot = imp.snapshot.borrow().clone();
        let id = id.map(PresetId::new);
        let (Some(snapshot), Some(id)) = (snapshot, id) else {
            return;
        };
        if snapshot.presets.find(&id).is_none() {
            return;
        }
        imp.eq_editor_page.open(&snapshot, &id);
        if imp.navigation.visible_page_tag().as_deref() != Some(EDITOR_TAG) {
            imp.navigation.push_by_tag(EDITOR_TAG);
        }
    }

    fn show_page(&self, tag: Option<&str>) {
        let navigation = &self.imp().navigation;
        if let Some(page) = tag.and_then(|tag| navigation.find_page(tag)) {
            navigation.push(&page);
        }
    }

    fn render_app_presets(&self, snapshot: &Snapshot) {
        self.imp()
            .app_presets_page
            .render(&app_presets(snapshot), &add_rule_dialog(snapshot));
    }

    fn render_main(&self, model: &HeadsetPageModel) {
        match model {
            HeadsetPageModel::Status(kind, page) => self.show_status(*kind, page),
            HeadsetPageModel::Headset(headset) => self.show_headset(headset),
        }
    }

    fn show_status(&self, kind: StatusKind, page: &StatusPageModel) {
        let imp = self.imp();
        imp.status_view.render(page, status_action(kind));
        imp.banner.set_revealed(false);
        imp.main_stack.set_visible_child_name("status");
    }

    fn show_headset(&self, model: &HeadsetModel) {
        let imp = self.imp();
        imp.headset_page.render(model);
        self.show_banner(model.banner.as_ref());
        imp.main_stack.set_visible_child_name("headset");
        if let Some(toast) = imp
            .toast_feed
            .toast_for_new_failure(model.failure_toast.as_ref(), &imp.sink)
        {
            imp.toasts.add_toast(toast);
        }
    }

    fn show_banner(&self, banner: Option<&BannerModel>) {
        let view = &self.imp().banner;
        view.set_revealed(banner.is_some());
        if let Some(banner) = banner {
            view.set_title(&resolve(&banner.title));
            view.set_button_label(banner.button.as_ref().map(resolve).as_deref());
            view.set_action_name(banner.button.is_some().then_some(PERMISSION_ACTION));
        }
    }
}

const fn status_action(kind: StatusKind) -> Option<&'static str> {
    match kind {
        StatusKind::PermissionRequired => Some(PERMISSION_ACTION),
        StatusKind::NoHeadset => None,
    }
}

fn refresh_battery_of(window: &glib::WeakRef<Window>) -> glib::ControlFlow {
    match window.upgrade() {
        Some(window) => {
            window.refresh_battery();
            glib::ControlFlow::Continue
        }
        None => glib::ControlFlow::Break,
    }
}

fn current_time() -> UnixSeconds {
    u64::try_from(glib::real_time() / MICROS_PER_SECOND).map_or(UnixSeconds(0), UnixSeconds)
}
