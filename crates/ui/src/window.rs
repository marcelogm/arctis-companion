use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::companion::Snapshot;
use arctis_core::intent::Intent;
use arctis_core::view::headset_page::{
    BannerModel, HeadsetModel, HeadsetPageModel, StatusKind, headset_page,
};
use arctis_core::view::preferences::{Background, preferences};
use arctis_core::view::status_page::StatusPageModel;
use gtk::{gio, glib};

use crate::preferences_dialog::PreferencesDialog;
use crate::text::resolve;

const PERMISSION_ACTION: &str = "win.permission-instructions";

mod imp {
    use std::cell::{Cell, OnceCell, RefCell};
    use std::rc::Rc;

    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use arctis_core::companion::Snapshot;
    use arctis_core::view::preferences::Background;
    use gtk::{gdk, glib};

    use crate::app_dialogs::{about_dialog, shortcuts_dialog};
    use crate::headset_page::HeadsetPage;
    use crate::intents::IntentSink;
    use crate::permission_dialog::PermissionDialog;
    use crate::preferences_dialog::PreferencesDialog;
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
        pub sink: IntentSink,
        pub toast_feed: ToastFeed,
        pub snapshot: RefCell<Option<Rc<Snapshot>>>,
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
            self.render_preferences();
        });
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

    fn show_page(&self, tag: Option<&str>) {
        let navigation = &self.imp().navigation;
        if let Some(page) = tag.and_then(|tag| navigation.find_page(tag)) {
            navigation.push(&page);
        }
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
