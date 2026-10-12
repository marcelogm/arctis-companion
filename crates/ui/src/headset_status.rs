use adw::subclass::prelude::*;
use arctis_core::view::headset_page::HeaderModel;
use gtk::accessible::Property;
use gtk::glib;
use gtk::prelude::*;

use crate::text::{optional, resolve};
use crate::widget_values::show_label;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::glib;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/marcelogm/ArctisCompanion/ui/headset_status.ui")]
    pub struct HeadsetStatus {
        #[template_child]
        pub level_icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub battery_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub route_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub connection_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub spinner: TemplateChild<adw::Spinner>,
        #[template_child]
        pub subtitle_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HeadsetStatus {
        const NAME: &'static str = "ArctisHeadsetStatus";
        type Type = super::HeadsetStatus;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for HeadsetStatus {}
    impl WidgetImpl for HeadsetStatus {}
    impl BinImpl for HeadsetStatus {}
}

glib::wrapper! {
    pub struct HeadsetStatus(ObjectSubclass<imp::HeadsetStatus>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl HeadsetStatus {
    pub fn render(&self, model: &HeaderModel) {
        let imp = self.imp();
        imp.level_icon.set_icon_name(Some(&model.icon));
        show_label(&imp.battery_label, model.battery.as_ref());
        imp.route_label.set_label(&resolve(&model.route));
        imp.connection_box.set_visible(model.subtitle.is_some());
        imp.subtitle_label
            .set_label(&optional(model.subtitle.as_ref()));
        imp.spinner.set_visible(model.busy);
        self.describe(&resolve(&model.description));
    }

    fn describe(&self, description: &str) {
        self.update_property(&[Property::Description(description)]);
    }
}
