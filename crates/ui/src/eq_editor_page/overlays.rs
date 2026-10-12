use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::eq::reference::Overlays;
use arctis_core::intent::Intent;
use arctis_core::view::eq_editor::EqEditorModel;
use gtk::{gio, glib};

use super::EqEditorPage;

const GROUP: &str = "reference";
const HEADSET: &str = "headset";
const TARGET: &str = "target";

impl EqEditorPage {
    pub(super) fn connect_overlays(&self) {
        let shown = Overlays::default();
        let headset = gio::SimpleAction::new_stateful(HEADSET, None, &shown.headset.to_variant());
        headset.connect_change_state(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_, value| {
                if let Some(on) = value.and_then(glib::Variant::get::<bool>) {
                    page.choose_overlays(page.overlays().with_headset(on));
                }
            }
        ));
        let target = gio::SimpleAction::new_stateful(
            TARGET,
            Some(glib::VariantTy::STRING),
            &shown.target_setting().to_variant(),
        );
        target.connect_change_state(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_, value| {
                if let Some(setting) = value.and_then(glib::Variant::str) {
                    page.choose_overlays(page.overlays().with_target_setting(setting));
                }
            }
        ));
        let group = gio::SimpleActionGroup::new();
        group.add_action(&headset);
        group.add_action(&target);
        self.insert_action_group(GROUP, Some(&group));
        self.imp().overlay_actions.replace(Some(group));
    }

    pub(super) fn overlays(&self) -> Overlays {
        self.imp().overlays.get()
    }

    pub(super) fn show_overlays(&self, model: &EqEditorModel) {
        let imp = self.imp();
        imp.legend.render(&model.graph.legend);
        let overlays = self.overlays();
        let group = imp.overlay_actions.borrow();
        let Some(group) = group.as_ref() else {
            return;
        };
        set_state(group, HEADSET, &overlays.headset.to_variant());
        set_state(group, TARGET, &overlays.target_setting().to_variant());
    }

    fn choose_overlays(&self, overlays: Overlays) {
        if self.imp().rendering.get() || overlays == self.overlays() {
            return;
        }
        self.imp().overlays.set(overlays);
        self.emit(Intent::SetOverlays(overlays));
        self.render_model();
    }
}

fn set_state(group: &gio::SimpleActionGroup, name: &str, state: &glib::Variant) {
    let action = group
        .lookup_action(name)
        .and_then(|action| action.downcast::<gio::SimpleAction>().ok());
    if let Some(action) = action {
        action.set_state(state);
    }
}
