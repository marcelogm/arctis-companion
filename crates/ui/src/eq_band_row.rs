use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::view::eq_editor::{BandModel, kind_at, kind_items, kind_position};
use arctis_core::view::preset_edits::BandEdit;
use gtk::glib;

use crate::text::resolve;
use crate::widget_values::{index, show_choices, while_rendering};

type EditHandler = Rc<dyn Fn(BandEdit)>;
type RemoveHandler = Rc<dyn Fn()>;

mod imp {
    use std::cell::{Cell, RefCell};

    use adw::subclass::prelude::*;
    use gtk::glib;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/marcelogm/ArctisCompanion/ui/eq_band_row.ui")]
    pub struct EqBandRow {
        #[template_child]
        pub kind: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub frequency: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub gain: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub q: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub enabled: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub remove: TemplateChild<adw::ButtonRow>,
        pub rendering: Cell<bool>,
        pub on_edit: RefCell<Option<super::EditHandler>>,
        pub on_remove: RefCell<Option<super::RemoveHandler>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EqBandRow {
        const NAME: &'static str = "ArctisEqBandRow";
        type Type = super::EqBandRow;
        type ParentType = adw::ExpanderRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for EqBandRow {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().connect_controls();
        }
    }

    impl WidgetImpl for EqBandRow {}
    impl ListBoxRowImpl for EqBandRow {}
    impl PreferencesRowImpl for EqBandRow {}
    impl ExpanderRowImpl for EqBandRow {}
}

glib::wrapper! {
    pub struct EqBandRow(ObjectSubclass<imp::EqBandRow>)
        @extends adw::ExpanderRow, adw::PreferencesRow, gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for EqBandRow {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl EqBandRow {
    pub fn render(&self, band: &BandModel, editable: bool) {
        let imp = self.imp();
        while_rendering(&imp.rendering, || {
            self.set_title(&resolve(&band.title));
            let kinds: Vec<String> = kind_items().iter().map(resolve).collect();
            show_choices(&imp.kind, &kinds, kind_position(band.kind));
            imp.frequency.set_value(f64::from(band.hz.get()));
            imp.gain.set_value(band.gain.db());
            imp.q.set_value(band.q.value());
            imp.enabled.set_active(band.enabled);
            self.set_editable(editable, band.gain_sensitive);
        });
    }

    pub fn connect_edit(&self, handler: impl Fn(BandEdit) + 'static) {
        self.imp().on_edit.replace(Some(Rc::new(handler)));
    }

    pub fn connect_remove(&self, handler: impl Fn() + 'static) {
        self.imp().on_remove.replace(Some(Rc::new(handler)));
    }

    fn set_editable(&self, editable: bool, gain_sensitive: bool) {
        let imp = self.imp();
        imp.kind.set_sensitive(editable);
        imp.frequency.set_sensitive(editable);
        imp.gain.set_sensitive(editable && gain_sensitive);
        imp.q.set_sensitive(editable);
        imp.enabled.set_sensitive(editable);
        imp.remove.set_visible(editable);
    }

    fn connect_controls(&self) {
        let imp = self.imp();
        imp.kind.connect_selected_notify(glib::clone!(
            #[weak(rename_to = row)]
            self,
            move |kind| row
                .report_edit(index(kind.selected()).and_then(kind_at).map(BandEdit::Kind))
        ));
        self.connect_spin(&imp.frequency, BandEdit::Frequency);
        self.connect_spin(&imp.gain, BandEdit::Gain);
        self.connect_spin(&imp.q, BandEdit::Q);
        imp.enabled.connect_active_notify(glib::clone!(
            #[weak(rename_to = row)]
            self,
            move |switch| row.report_edit(Some(BandEdit::Enabled(switch.is_active())))
        ));
        imp.remove.connect_activated(glib::clone!(
            #[weak(rename_to = row)]
            self,
            move |_| row.report_removal()
        ));
    }

    fn connect_spin(&self, spin: &adw::SpinRow, edit: fn(f64) -> BandEdit) {
        spin.connect_value_notify(glib::clone!(
            #[weak(rename_to = row)]
            self,
            move |spin| row.report_edit(Some(edit(spin.value())))
        ));
    }

    fn report_edit(&self, edit: Option<BandEdit>) {
        let imp = self.imp();
        if imp.rendering.get() {
            return;
        }
        let handler = imp.on_edit.borrow().clone();
        if let (Some(handler), Some(edit)) = (handler, edit) {
            handler(edit);
        }
    }

    fn report_removal(&self) {
        let handler = self.imp().on_remove.borrow().clone();
        if let Some(handler) = handler {
            handler();
        }
    }
}
