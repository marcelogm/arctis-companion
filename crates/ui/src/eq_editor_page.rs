mod graph;
mod header;
mod overlays;

use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::companion::Snapshot;
use arctis_core::eq::preset::{Preset, PresetId};
use arctis_core::intent::Intent;
use arctis_core::view::eq_editor::{EqEditorModel, PreviewState, eq_editor_with};
use arctis_core::view::preset_edits::{
    BandEdit, WorkingCopy, apply_edit, remove_band, saved_toast,
};
use gtk::glib;

use crate::eq_band_row::EqBandRow;
use crate::services::Services;
use crate::text::resolve;
use crate::widget_values::while_rendering;

pub const EDITOR_TAG: &str = "eq-editor";

const EDITABLE_ACTIONS: [&str; 3] = ["editor.rename", "editor.export", "editor.delete"];

mod imp {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use adw::subclass::prelude::*;
    use arctis_core::companion::Snapshot;
    use arctis_core::eq::reference::Overlays;
    use arctis_core::view::preset_edits::{WorkingCopy, add_band};
    use gtk::glib::prelude::StaticTypeExt;
    use gtk::{gio, glib};

    use crate::eq_band_row::EqBandRow;
    use crate::eq_graph::{EqGraph, EqLegend};
    use crate::services::Services;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/marcelogm/ArctisCompanion/ui/eq_editor_page.ui")]
    pub struct EqEditorPage {
        #[template_child]
        pub menu: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub duplicate: TemplateChild<gtk::Button>,
        #[template_child]
        pub preview: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub legend: TemplateChild<EqLegend>,
        #[template_child]
        pub graph: TemplateChild<EqGraph>,
        #[template_child]
        pub bands: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub add_band: TemplateChild<gtk::Button>,
        pub services: RefCell<Option<Services>>,
        pub snapshot: RefCell<Option<Rc<Snapshot>>>,
        pub working: RefCell<Option<WorkingCopy>>,
        pub dragged: RefCell<Option<arctis_core::eq::preset::Preset>>,
        pub band_rows: RefCell<Vec<EqBandRow>>,
        pub undo_toast: RefCell<Option<adw::Toast>>,
        pub rendering: Cell<bool>,
        pub picked: Cell<bool>,
        pub overlays: Cell<Overlays>,
        pub overlay_actions: RefCell<Option<gio::SimpleActionGroup>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EqEditorPage {
        const NAME: &'static str = "ArctisEqEditorPage";
        type Type = super::EqEditorPage;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            EqGraph::ensure_type();
            EqLegend::ensure_type();
            klass.bind_template();
            klass.install_action("editor.add-band", None, |page, _, _| {
                page.save_change(add_band);
            });
            klass.install_action("editor.duplicate", None, |page, _, _| {
                page.duplicate_preset();
            });
            klass.install_action("editor.rename", None, |page, _, _| page.ask_rename());
            klass.install_action("editor.export", None, |page, _, _| page.export_preset());
            klass.install_action("editor.delete", None, |page, _, _| page.delete_preset());
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for EqEditorPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().connect_preview();
            self.obj().connect_graph();
            self.obj().connect_overlays();
        }
    }

    impl WidgetImpl for EqEditorPage {
        fn unmap(&self) {
            self.parent_unmap();
            self.obj().end_preview();
        }
    }

    impl NavigationPageImpl for EqEditorPage {
        fn hidden(&self) {
            self.parent_hidden();
            self.obj().end_preview();
        }
    }
}

glib::wrapper! {
    pub struct EqEditorPage(ObjectSubclass<imp::EqEditorPage>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for EqEditorPage {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl EqEditorPage {
    pub fn set_services(&self, services: Services) {
        self.imp().services.replace(Some(services));
    }

    pub fn open(&self, snapshot: &Rc<Snapshot>, id: &PresetId) {
        self.open_working(snapshot, snapshot.presets.find(id).cloned());
    }

    pub fn open_new(&self, snapshot: &Rc<Snapshot>, preset: Preset) {
        self.open_working(snapshot, Some(preset));
    }

    fn open_working(&self, snapshot: &Rc<Snapshot>, preset: Option<Preset>) {
        self.end_preview();
        self.dismiss_undo_toast();
        self.imp().dragged.replace(None);
        self.imp().working.replace(preset.map(WorkingCopy::opened));
        self.render(snapshot);
    }

    pub fn render(&self, snapshot: &Rc<Snapshot>) {
        let imp = self.imp();
        imp.snapshot.replace(Some(Rc::clone(snapshot)));
        imp.overlays.set(snapshot.presets.overlays);
        if let Some(working) = imp.working.borrow_mut().as_mut() {
            working.published(snapshot);
        }
        self.render_model();
    }

    fn render_model(&self) {
        let imp = self.imp();
        let snapshot = imp.snapshot.borrow().clone();
        let preset = imp.dragged.borrow().clone().or_else(|| self.preset());
        let (Some(snapshot), Some(preset)) = (snapshot, preset) else {
            return;
        };
        let model = eq_editor_with(&snapshot, &preset, self.overlays());
        while_rendering(&imp.rendering, || {
            self.set_title(&resolve(&model.title));
            self.show_header(&model);
            self.show_overlays(&model);
            imp.graph.render(&model.graph, model.editable);
            self.show_bands(&model);
        });
    }

    fn show_header(&self, model: &EqEditorModel) {
        let imp = self.imp();
        imp.duplicate.set_visible(!model.editable);
        imp.menu.set_visible(model.editable);
        imp.add_band.set_visible(model.editable);
        self.show_preview(model);
        self.action_set_enabled("editor.add-band", model.editable && model.can_add_band);
        self.action_set_enabled("editor.duplicate", !model.editable);
        for action in EDITABLE_ACTIONS {
            self.action_set_enabled(action, model.editable);
        }
    }

    fn show_preview(&self, model: &EqEditorModel) {
        let imp = self.imp();
        let available = model.preview.is_available();
        imp.preview.set_visible(available);
        let picked = model.preview == PreviewState::Picked;
        let pick_ended = imp.picked.replace(picked) && !picked;
        if pick_ended || !available {
            imp.preview.set_active(false);
        }
    }

    fn show_bands(&self, model: &EqEditorModel) {
        self.sync_band_rows(model.bands.len());
        let rows = self.imp().band_rows.borrow();
        for (row, band) in rows.iter().zip(&model.bands) {
            row.render(band, model.editable);
        }
    }

    fn sync_band_rows(&self, count: usize) {
        let imp = self.imp();
        let mut rows = imp.band_rows.borrow_mut();
        let keep = count.min(rows.len());
        for row in rows.drain(keep..) {
            imp.bands.remove(&row);
        }
        while rows.len() < count {
            let row = self.band_row();
            imp.bands.add(&row);
            rows.push(row);
        }
    }

    fn band_row(&self) -> EqBandRow {
        let row = EqBandRow::default();
        row.set_widget_name("band-row");
        row.connect_edit(glib::clone!(
            #[weak(rename_to = page)]
            self,
            #[weak]
            row,
            move |edit| page.edit_band(&row, edit)
        ));
        row.connect_remove(glib::clone!(
            #[weak(rename_to = page)]
            self,
            #[weak]
            row,
            move || page.remove_band_row(&row)
        ));
        self.follow_row(&row);
        row
    }

    fn band_index(&self, row: &EqBandRow) -> Option<usize> {
        let rows = self.imp().band_rows.borrow();
        rows.iter().position(|known| known == row)
    }

    fn edit_band(&self, row: &EqBandRow, edit: BandEdit) {
        if let Some(index) = self.band_index(row) {
            self.save_change(|preset| apply_edit(preset, index, edit));
        }
    }

    fn remove_band_row(&self, row: &EqBandRow) {
        let Some(index) = self.band_index(row) else {
            return;
        };
        let imp = self.imp();
        imp.band_rows.borrow_mut().remove(index);
        imp.bands.remove(row);
        self.save_change(|preset| remove_band(preset, index));
        self.focus_band(index);
    }

    fn focus_band(&self, index: usize) {
        let imp = self.imp();
        let next = imp.band_rows.borrow().get(index).cloned();
        match next {
            Some(row) => row.grab_focus(),
            None => imp.add_band.grab_focus(),
        };
    }

    fn save_change(&self, change: impl FnOnce(&Preset) -> Option<Preset>) {
        let applied = self
            .imp()
            .working
            .borrow_mut()
            .as_mut()
            .and_then(|working| working.apply(change));
        let Some(applied) = applied else {
            self.render_model();
            return;
        };
        self.emit(Intent::SavePreset(applied.preset.clone()));
        self.render_model();
        if applied.starts_burst {
            self.offer_undo(&applied.preset);
        }
    }

    fn undo(&self) {
        if !self.is_published() {
            return;
        }
        let restored = self
            .imp()
            .working
            .borrow_mut()
            .as_mut()
            .and_then(WorkingCopy::undo);
        if let Some(restored) = restored {
            self.emit(Intent::SavePreset(restored));
            self.render_model();
        }
    }

    fn offer_undo(&self, preset: &Preset) {
        let Some(services) = self.services() else {
            return;
        };
        let toast = services.undo_toast(
            &saved_toast(preset),
            glib::clone!(
                #[weak(rename_to = page)]
                self,
                move || page.undo()
            ),
        );
        toast.connect_dismissed(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |toast| page.undo_toast_dismissed(toast)
        ));
        self.imp().undo_toast.replace(Some(toast));
    }

    fn undo_toast_dismissed(&self, dismissed: &adw::Toast) {
        let imp = self.imp();
        if imp.undo_toast.borrow().as_ref() != Some(dismissed) {
            return;
        }
        imp.undo_toast.replace(None);
        if let Some(working) = imp.working.borrow_mut().as_mut() {
            working.burst_ended();
        }
    }

    fn dismiss_undo_toast(&self) {
        if let Some(toast) = self.imp().undo_toast.take() {
            toast.dismiss();
        }
    }

    fn is_published(&self) -> bool {
        let snapshot = self.imp().snapshot.borrow().clone();
        let (Some(snapshot), Some(preset)) = (snapshot, self.preset()) else {
            return false;
        };
        snapshot.presets.find(preset.id()).is_some()
    }

    fn preset(&self) -> Option<Preset> {
        let working = self.imp().working.borrow();
        working.as_ref().map(|working| working.preset().clone())
    }

    fn emit(&self, intent: Intent) {
        if let Some(services) = self.services() {
            services.sink.emit(intent);
        }
    }

    fn services(&self) -> Option<Services> {
        self.imp().services.borrow().clone()
    }
}
