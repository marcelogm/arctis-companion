use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::intent::Intent;
use arctis_core::settings::headset_settings::Setting;
use arctis_core::view::headset_page::{HeadsetModel, SettingsArea};
use arctis_core::view::headset_rows::HeadsetGroupsModel;
use gtk::glib;

use crate::headset_groups::{DeviceRows, show_choice, show_device, show_scale, show_switch};
use crate::intents::IntentSink;
use crate::text::resolve;
use crate::widget_values::index;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::glib;
    use gtk::glib::prelude::StaticTypeExt;

    use crate::equalizer_group::EqualizerGroup;
    use crate::headset_status::HeadsetStatus;
    use crate::scale_row::ScaleRow;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/marcelogm/ArctisCompanion/ui/headset_page.ui")]
    pub struct HeadsetPage {
        #[template_child]
        pub status: TemplateChild<HeadsetStatus>,
        #[template_child]
        pub equalizer: TemplateChild<EqualizerGroup>,
        #[template_child]
        pub bluetooth_only_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub microphone_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub mic_volume: TemplateChild<ScaleRow>,
        #[template_child]
        pub sidetone: TemplateChild<ScaleRow>,
        #[template_child]
        pub mute_light: TemplateChild<ScaleRow>,
        #[template_child]
        pub sound_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub volume_limiter: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub wireless_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub extended_range: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub auto_power_off: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub device_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub model_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub support_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub firmware_row: TemplateChild<adw::ActionRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HeadsetPage {
        const NAME: &'static str = "ArctisHeadsetPage";
        type Type = super::HeadsetPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            HeadsetStatus::ensure_type();
            EqualizerGroup::ensure_type();
            ScaleRow::ensure_type();
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for HeadsetPage {}
    impl WidgetImpl for HeadsetPage {}
    impl BinImpl for HeadsetPage {}
}

glib::wrapper! {
    pub struct HeadsetPage(ObjectSubclass<imp::HeadsetPage>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl HeadsetPage {
    pub fn render(&self, model: &HeadsetModel) {
        let imp = self.imp();
        imp.status.render(&model.header);
        imp.equalizer.render(&model.equalizer);
        self.render_area(&model.settings);
    }

    pub fn connect_intents(&self, sink: &IntentSink) {
        let imp = self.imp();
        imp.equalizer.connect_intents(sink);
        for row in [&imp.mic_volume, &imp.sidetone, &imp.mute_light] {
            row.connect_intents(sink);
        }
        connect_switch(&imp.volume_limiter, Setting::VolumeLimiter, sink);
        connect_switch(&imp.extended_range, Setting::WirelessMode, sink);
        let sink = sink.clone();
        imp.auto_power_off.connect_selected_notify(move |row| {
            sink.emit_some(index(row.selected()).and_then(Intent::from_auto_power_off_choice));
        });
    }

    fn render_area(&self, area: &SettingsArea) {
        let imp = self.imp();
        let note = match area {
            SettingsArea::BluetoothOnly(text) => Some(resolve(text)),
            SettingsArea::Groups(_) | SettingsArea::Hidden => None,
        };
        imp.bluetooth_only_group.set_visible(note.is_some());
        imp.bluetooth_only_group.set_description(note.as_deref());
        let groups = match area {
            SettingsArea::Groups(groups) => Some(groups.as_ref()),
            SettingsArea::BluetoothOnly(_) | SettingsArea::Hidden => None,
        };
        self.render_groups(groups);
    }

    fn render_groups(&self, groups: Option<&HeadsetGroupsModel>) {
        let imp = self.imp();
        let microphone = [
            show_scale(&imp.mic_volume, groups.and_then(|g| g.mic_volume.as_ref())),
            show_scale(&imp.sidetone, groups.and_then(|g| g.sidetone.as_ref())),
            show_scale(&imp.mute_light, groups.and_then(|g| g.mute_light.as_ref())),
        ];
        imp.microphone_group.set_visible(microphone.contains(&true));
        let limiter = groups.and_then(|g| g.volume_limiter.as_ref());
        imp.sound_group
            .set_visible(show_switch(&imp.volume_limiter, limiter));
        let range = groups.and_then(|g| g.extended_range.as_ref());
        imp.wireless_group
            .set_visible(show_switch(&imp.extended_range, range));
        show_choice(
            &imp.auto_power_off,
            groups.and_then(|g| g.auto_power_off.as_ref()),
        );
        self.render_device(groups);
        self.set_settings_sensitive(groups.is_none_or(|g| g.sensitive));
    }

    fn render_device(&self, groups: Option<&HeadsetGroupsModel>) {
        let imp = self.imp();
        imp.device_group.set_visible(groups.is_some());
        let rows = DeviceRows {
            model: &imp.model_row,
            support: &imp.support_row,
            firmware: &imp.firmware_row,
        };
        if let Some(groups) = groups {
            show_device(&rows, &groups.device);
        }
    }

    fn set_settings_sensitive(&self, sensitive: bool) {
        let imp = self.imp();
        let settings: [&gtk::Widget; 4] = [
            imp.microphone_group.upcast_ref(),
            imp.sound_group.upcast_ref(),
            imp.wireless_group.upcast_ref(),
            imp.auto_power_off.upcast_ref(),
        ];
        for widget in settings {
            widget.set_sensitive(sensitive);
        }
    }
}

fn connect_switch(row: &adw::SwitchRow, setting: Setting, sink: &IntentSink) {
    let sink = sink.clone();
    row.connect_active_notify(move |row| {
        sink.emit_some(Intent::from_switch(setting, row.is_active()));
    });
}
