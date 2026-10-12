use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::intent::Intent;
use arctis_core::l10n::msgid;
use arctis_core::view::preset_edits::{WorkingCopy, copy_name, duplicate, renamed};
use arctis_core::view::preset_files::{export_failed, export_file_name, export_text};
use arctis_core::view::presets::deleted_toast;
use arctis_core::view::text::Text;
use gtk::accessible::Property;
use gtk::glib;

use super::EqEditorPage;
use crate::preset_files::Export;
use crate::services::Services;
use crate::text::resolve;

const CANCEL: &str = "cancel";
const RENAME: &str = "rename";

impl EqEditorPage {
    pub fn rename_to(&self, name: &str) {
        self.save_change(|preset| renamed(preset, name));
    }

    pub(super) fn duplicate_preset(&self) {
        let imp = self.imp();
        let snapshot = imp.snapshot.borrow().clone();
        let (Some(snapshot), Some(source)) = (snapshot, self.preset()) else {
            return;
        };
        let copy = duplicate(&snapshot, &source, &resolve(&copy_name(&source)));
        self.end_preview();
        self.dismiss_undo_toast();
        imp.working.replace(Some(WorkingCopy::opened(copy.clone())));
        self.emit(Intent::SavePreset(copy));
        self.render_model();
    }

    pub(super) fn ask_rename(&self) {
        let Some(preset) = self.preset() else {
            return;
        };
        let (dialog, entry) = rename_dialog(preset.name());
        dialog.connect_response(
            Some(RENAME),
            glib::clone!(
                #[weak(rename_to = page)]
                self,
                #[weak]
                entry,
                move |_, _| page.rename_to(&entry.text())
            ),
        );
        dialog.present(Some(self));
    }

    pub(super) fn export_preset(&self) {
        let (Some(services), Some(preset)) = (self.services(), self.preset()) else {
            return;
        };
        let failed = export_failed(&preset);
        let export = Export {
            suggested_name: export_file_name(preset.name()),
            render: Box::new(move |file_name: &str| export_text(&preset, file_name)),
            done: report_failure(services.clone(), failed),
        };
        services.files.save(self.upcast_ref(), export);
    }

    pub(super) fn delete_preset(&self) {
        let snapshot = self.imp().snapshot.borrow().clone();
        let (Some(services), Some(snapshot), Some(preset)) =
            (self.services(), snapshot, self.preset())
        else {
            return;
        };
        let id = preset.id().clone();
        let sink = services.sink.clone();
        let title = deleted_toast(&snapshot, &preset);
        services.urgent_undo_toast(&title, move || {
            sink.emit(Intent::SavePreset(preset.clone()));
        });
        self.end_preview();
        self.activate_action("navigation.pop", None).ok();
        services.sink.emit(Intent::DeletePreset(id));
    }

    pub(super) fn connect_preview(&self) {
        self.imp().preview.connect_toggled(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |toggle| page.preview_toggled(toggle.is_active())
        ));
    }

    pub(super) fn end_preview(&self) {
        let preview = &self.imp().preview;
        if preview.is_active() {
            preview.set_active(false);
        }
    }

    fn preview_toggled(&self, active: bool) {
        if self.imp().rendering.get() {
            return;
        }
        let intent = if active {
            self.preset()
                .map(|preset| Intent::StartPreview(preset.id().clone()))
        } else {
            Some(Intent::EndPreview)
        };
        if let Some(intent) = intent {
            self.emit(intent);
        }
    }
}

fn report_failure(services: Services, failed: Text) -> Box<dyn FnOnce(bool)> {
    Box::new(move |written| {
        if !written {
            services.toast(&failed);
        }
    })
}

fn rename_dialog(name: &str) -> (adw::AlertDialog, gtk::Entry) {
    let entry = gtk::Entry::builder()
        .text(name)
        .activates_default(true)
        .build();
    entry.update_property(&[Property::Label(&resolve(&Text::plain(msgid("Name"))))]);
    let dialog = adw::AlertDialog::builder()
        .heading(resolve(&Text::plain(msgid("Rename Preset"))))
        .extra_child(&entry)
        .default_response(RENAME)
        .close_response(CANCEL)
        .build();
    dialog.add_responses(&[
        (CANCEL, &resolve(&Text::plain(msgid("_Cancel")))),
        (RENAME, &resolve(&Text::plain(msgid("_Rename")))),
    ]);
    dialog.set_response_appearance(RENAME, adw::ResponseAppearance::Suggested);
    (dialog, entry)
}
