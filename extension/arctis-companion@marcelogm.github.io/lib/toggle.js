import GObject from 'gi://GObject';

import {gettext as _} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import {QuickMenuToggle} from 'resource:///org/gnome/shell/ui/quickSettings.js';

import {reportFailure} from './failures.js';
import {HIDDEN, automaticPresetsAfterClick} from './presentation.js';

const HEADSET_ICON = 'audio-headset-symbolic';

export const AppPresetsToggle = GObject.registerClass(
class AppPresetsToggle extends QuickMenuToggle {
    _init(client) {
        super._init({title: _('App Presets'), iconName: HEADSET_ICON, toggleMode: false, visible: false});
        this._client = client;
        this._presentation = HIDDEN;
        this._presets = new PopupMenu.PopupMenuSection();
        this.menu.addMenuItem(this._presets);
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        this.menu.addAction(_('Headset Settings'), () => this._openSettings());
        this.connect('clicked', () => this._toggleAutomaticPresets());
    }

    render(presentation) {
        this._presentation = presentation;
        this.visible = presentation.visible;
        if (presentation.visible)
            this._renderVisible(presentation);
    }

    _renderVisible({title, subtitle, checked, sensitive, header, items}) {
        this.set({title, subtitle, checked});
        this._switchButton().reactive = sensitive;
        this.menu.setHeader(header.icon, header.title, header.subtitle);
        this._presets.removeAll();
        items.forEach(item => this._presets.addMenuItem(this._presetItem(item)));
    }

    _switchButton() {
        return this.get_child().get_first_child();
    }

    _presetItem({id, label, checked, sensitive}) {
        const item = new PopupMenu.PopupMenuItem(label);
        item.setOrnament(checked ? PopupMenu.Ornament.CHECK : PopupMenu.Ornament.NONE);
        item.setSensitive(sensitive);
        item.connect('activate', () => this._client.activatePreset(id).catch(reportFailure));
        return item;
    }

    _toggleAutomaticPresets() {
        const next = automaticPresetsAfterClick(this._presentation);
        if (next !== null)
            this._client.setAutomaticPresets(next).catch(reportFailure);
    }

    _openSettings() {
        Main.panel.closeQuickSettings();
        this._client.activateApplication().catch(reportFailure);
    }
});
