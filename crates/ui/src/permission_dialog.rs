use gtk::glib;
use gtk::prelude::*;

pub const UDEV_RULE: &str =
    include_str!("../../../data/udev/70-arctis-companion.rules").trim_ascii_end();
pub const RELOAD_COMMAND: &str = "sudo udevadm control --reload-rules && sudo udevadm trigger";

mod imp {
    use adw::subclass::prelude::*;
    use gtk::glib;

    use super::{RELOAD_COMMAND, UDEV_RULE};

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/marcelogm/ArctisCompanion/ui/permission_dialog.ui")]
    pub struct PermissionDialog {
        #[template_child]
        pub rule_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub copy_rule: TemplateChild<gtk::Button>,
        #[template_child]
        pub command_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub copy_command: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PermissionDialog {
        const NAME: &'static str = "ArctisPermissionDialog";
        type Type = super::PermissionDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PermissionDialog {
        fn constructed(&self) {
            self.parent_constructed();
            self.rule_label.set_label(UDEV_RULE);
            self.command_label.set_label(RELOAD_COMMAND);
            super::copy_on_click(&self.copy_rule, UDEV_RULE);
            super::copy_on_click(&self.copy_command, RELOAD_COMMAND);
        }
    }

    impl WidgetImpl for PermissionDialog {}
    impl AdwDialogImpl for PermissionDialog {}
}

glib::wrapper! {
    pub struct PermissionDialog(ObjectSubclass<imp::PermissionDialog>)
        @extends adw::Dialog, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for PermissionDialog {
    fn default() -> Self {
        glib::Object::new()
    }
}

fn copy_on_click(button: &gtk::Button, text: &'static str) {
    button.connect_clicked(move |button| button.clipboard().set_text(text));
}
