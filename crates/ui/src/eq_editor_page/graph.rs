use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::view::preset_edits::{BandEdit, add_band_at, apply_edit};
use gtk::glib;

use super::EqEditorPage;
use crate::eq_band_row::EqBandRow;
use crate::eq_graph::GraphAction;

impl EqEditorPage {
    pub(super) fn connect_graph(&self) {
        self.imp().graph.connect_action(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |action| page.graph_action(action)
        ));
    }

    pub(super) fn follow_row(&self, row: &EqBandRow) {
        row.connect_expanded_notify(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |row| {
                if row.is_expanded() {
                    page.imp().graph.select(page.band_index(row));
                }
            }
        ));
    }

    fn graph_action(&self, action: GraphAction) {
        match action {
            GraphAction::Select(index) => self.reveal_band(index),
            GraphAction::Preview(index, edit) => self.preview_edit(index, edit),
            GraphAction::Commit(index, edit) => self.commit_edit(index, edit),
            GraphAction::Add { hz, db } => self.add_band_at(hz, db),
            GraphAction::Remove(index) => self.remove_from_graph(index),
        }
    }

    fn reveal_band(&self, index: usize) {
        for (at, row) in self.imp().band_rows.borrow().iter().enumerate() {
            row.set_expanded(at == index);
        }
    }

    fn preview_edit(&self, index: usize, edit: BandEdit) {
        let edited = self
            .preset()
            .and_then(|preset| apply_edit(&preset, index, edit));
        if let Some(edited) = edited {
            self.imp().dragged.replace(Some(edited));
            self.render_model();
        }
    }

    fn commit_edit(&self, index: usize, edit: BandEdit) {
        self.imp().dragged.replace(None);
        self.save_change(|preset| apply_edit(preset, index, edit));
    }

    fn add_band_at(&self, hz: f64, db: f64) {
        self.save_change(|preset| add_band_at(preset, hz, db));
        let count = self.imp().band_rows.borrow().len();
        if let Some(added) = count.checked_sub(1) {
            self.focus_handle(added);
            self.reveal_band(added);
        }
    }

    fn remove_from_graph(&self, index: usize) {
        let row = self.imp().band_rows.borrow().get(index).cloned();
        let Some(row) = row else {
            return;
        };
        self.remove_band_row(&row);
        let left = self.imp().band_rows.borrow().len();
        if let Some(last) = left.checked_sub(1) {
            self.focus_handle(index.min(last));
        }
    }

    fn focus_handle(&self, index: usize) {
        let graph = &self.imp().graph;
        graph.select(Some(index));
        if let Some(handle) = graph.handles().get(index) {
            handle.grab_focus();
        }
    }
}
