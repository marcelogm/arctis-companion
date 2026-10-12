import GObject from 'gi://GObject';

import {gettext as _} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import {QuickMenuToggle, QuickToggle} from 'resource:///org/gnome/shell/ui/quickSettings.js';

import {reportFailure} from './failures.js';
import {HEADSET_ICON, HIDDEN, automaticPresetsAfterClick} from './presentation.js';

export const AppPresetsToggle = GObject.registerClass(
class AppPresetsToggle extends QuickMenuToggle {
    _init(client) {
        super._init({title: HIDDEN.title, iconName: HEADSET_ICON, toggleMode: false, visible: false});
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
        if (presentation.visible)
            this._renderVisible(presentation);
        else
            this._hide();
    }

    _hide() {
        this.menu.close();
        this.visible = false;
    }

    _renderVisible({title, subtitle, checked, sensitive, header, items}) {
        this.visible = true;
        this.set({title, subtitle, checked});
        this._setPrimaryReactive(sensitive);
        this.menu.setHeader(header.icon, header.title, header.subtitle);
        this._presets.removeAll();
        items.forEach(item => this._presets.addMenuItem(this._presetItem(item)));
    }

    _setPrimaryReactive(reactive) {
        const button = this._primaryButton();
        if (button === null)
            reportFailure(new Error('quick toggle button not found'));
        else
            button.reactive = reactive;
    }

    _primaryButton() {
        return this.get_child().get_children().find(child => child instanceof QuickToggle) ?? null;
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
