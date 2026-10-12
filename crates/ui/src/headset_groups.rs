use adw::prelude::*;
use arctis_core::view::headset_rows::{ChoiceModel, DeviceModel, ScaleModel, SwitchModel};

use crate::scale_row::ScaleRow;
use crate::text::{optional, resolve};
use crate::widget_values::show_choices;

pub struct DeviceRows<'a> {
    pub model: &'a adw::ActionRow,
    pub support: &'a adw::ActionRow,
    pub firmware: &'a adw::ActionRow,
}

pub fn show_scale(row: &ScaleRow, model: Option<&ScaleModel>) -> bool {
    row.set_visible(model.is_some());
    if let Some(model) = model {
        row.render(model);
    }
    model.is_some()
}

pub fn show_switch(row: &adw::SwitchRow, model: Option<&SwitchModel>) -> bool {
    row.set_visible(model.is_some());
    if let Some(model) = model {
        row.set_title(&resolve(&model.title));
        row.set_subtitle(&optional(model.subtitle.as_ref()));
        row.set_active(model.active);
    }
    model.is_some()
}

pub fn show_choice(row: &adw::ComboRow, model: Option<&ChoiceModel>) {
    row.set_visible(model.is_some());
    if let Some(model) = model {
        row.set_title(&resolve(&model.title));
        row.set_subtitle(&optional(model.subtitle.as_ref()));
        let items: Vec<String> = model.items.iter().map(resolve).collect();
        show_choices(row, &items, model.selected);
    }
}

pub fn show_device(rows: &DeviceRows<'_>, model: &DeviceModel) {
    rows.model.set_subtitle(&resolve(&model.model));
    rows.support.set_visible(model.support.is_some());
    rows.support.set_title(&optional(model.support.as_ref()));
    rows.firmware.set_subtitle(&resolve(&model.firmware));
}
