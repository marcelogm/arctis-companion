use adw::subclass::prelude::*;
use arctis_core::view::status_page::StatusPageModel;
use gtk::glib;
use gtk::prelude::*;

use crate::text::{optional, resolve};

mod imp {
    use adw::subclass::prelude::*;
    use gtk::glib;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/marcelogm/ArctisCompanion/ui/status_pages.ui")]
    pub struct StatusView {
        #[template_child]
        pub page: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StatusView {
        const NAME: &'static str = "ArctisStatusView";
        type Type = super::StatusView;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for StatusView {}
    impl WidgetImpl for StatusView {}
    impl BinImpl for StatusView {}
}

glib::wrapper! {
    pub struct StatusView(ObjectSubclass<imp::StatusView>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl StatusView {
    pub fn render(&self, model: &StatusPageModel, button_action: Option<&str>) {
        let imp = self.imp();
        imp.page.set_icon_name(Some(model.icon));
        imp.page.set_title(&resolve(&model.title));
        imp.page.set_description(Some(&resolve(&model.description)));
        imp.button.set_visible(model.button.is_some());
        imp.button.set_label(&optional(model.button.as_ref()));
        imp.button.set_action_name(button_action);
    }
}
