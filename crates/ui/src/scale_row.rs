use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::intent::Intent;
use arctis_core::settings::headset_settings::Setting;
use arctis_core::view::headset_rows::{Mark, ScaleModel};
use gtk::accessible::Property;
use gtk::glib;

use crate::intents::IntentSink;
use crate::text::resolve;
use crate::widget_values::{nearest_step, show_label};

const QUIET_AND_LOUD: (&str, &str) = (
    "microphone-sensitivity-low-symbolic",
    "microphone-sensitivity-high-symbolic",
);

mod imp {
    use std::cell::RefCell;

    use adw::subclass::prelude::*;
    use arctis_core::view::headset_rows::ScaleModel;
    use gtk::glib;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/marcelogm/ArctisCompanion/ui/scale_row.ui")]
    pub struct ScaleRow {
        #[template_child]
        pub title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub subtitle_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub start_icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub end_icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub scale: TemplateChild<gtk::Scale>,
        pub model: RefCell<Option<ScaleModel>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScaleRow {
        const NAME: &'static str = "ArctisScaleRow";
        type Type = super::ScaleRow;
        type ParentType = adw::PreferencesRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ScaleRow {}
    impl WidgetImpl for ScaleRow {}
    impl ListBoxRowImpl for ScaleRow {}
    impl PreferencesRowImpl for ScaleRow {}
}

glib::wrapper! {
    pub struct ScaleRow(ObjectSubclass<imp::ScaleRow>)
        @extends adw::PreferencesRow, gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl ScaleRow {
    pub fn render(&self, model: &ScaleModel) {
        let imp = self.imp();
        let previous = imp.model.replace(Some(model.clone()));
        let title = resolve(&model.title);
        self.set_title(&title);
        imp.title_label.set_label(&title);
        show_label(&imp.subtitle_label, model.subtitle.as_ref());
        self.show_icons(model.setting);
        self.show_marks(
            previous.as_ref().map(|shown| shown.marks.as_slice()),
            &model.marks,
        );
        let adjustment = imp.scale.adjustment();
        adjustment.set_upper(f64::from(model.max));
        adjustment.set_value(f64::from(model.value));
        self.announce(&title, &resolve(&model.value_text));
    }

    pub fn connect_intents(&self, sink: &IntentSink) {
        self.imp().scale.connect_value_changed(glib::clone!(
            #[weak(rename_to = row)]
            self,
            #[strong]
            sink,
            move |scale| sink.emit_some(row.intent_for(scale.value()))
        ));
    }

    fn intent_for(&self, value: f64) -> Option<Intent> {
        let model = self.imp().model.borrow();
        let model = model.as_ref()?;
        Intent::from_scale(model.setting, nearest_step(value, model.max))
    }

    fn show_icons(&self, setting: Setting) {
        let imp = self.imp();
        let icons = (setting == Setting::MicVolume).then_some(QUIET_AND_LOUD);
        imp.start_icon.set_visible(icons.is_some());
        imp.end_icon.set_visible(icons.is_some());
        if let Some((quiet, loud)) = icons {
            imp.start_icon.set_icon_name(Some(quiet));
            imp.end_icon.set_icon_name(Some(loud));
        }
    }

    fn show_marks(&self, shown: Option<&[Mark]>, marks: &[Mark]) {
        let imp = self.imp();
        if shown == Some(marks) {
            return;
        }
        imp.scale.clear_marks();
        for mark in marks {
            let label = glib::markup_escape_text(&resolve(&mark.label));
            imp.scale.add_mark(
                f64::from(mark.value),
                gtk::PositionType::Bottom,
                Some(&label),
            );
        }
    }

    fn announce(&self, title: &str, value_text: &str) {
        let imp = self.imp();
        imp.scale
            .update_property(&[Property::Label(title), Property::ValueText(value_text)]);
    }
}
