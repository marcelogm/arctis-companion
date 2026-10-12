use adw::prelude::*;
use arctis_core::view::headset_rows::{ChoiceModel, ScaleModel, SwitchModel};

use crate::scale_row::ScaleRow;
use crate::text::{optional, resolve};
use crate::widget_values::{show_combo_row, show_if};

pub fn show_scale(row: &ScaleRow, model: Option<&ScaleModel>) -> bool {
    show_if(row, model, ScaleRow::render)
}

pub fn show_switch(row: &adw::SwitchRow, model: Option<&SwitchModel>) -> bool {
    show_if(row, model, |row, model| {
        row.set_title(&resolve(&model.title));
        row.set_subtitle(&optional(model.subtitle.as_ref()));
        row.set_active(model.active);
    })
}

pub fn show_choice(row: &adw::ComboRow, model: Option<&ChoiceModel>) -> bool {
    show_if(row, model, |row, model| {
        show_combo_row(
            row,
            (&model.title, model.subtitle.as_ref()),
            model.items.iter(),
            model.selected,
        );
    })
}
