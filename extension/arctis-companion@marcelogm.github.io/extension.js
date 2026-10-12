import Gio from 'gi://Gio';

import {Extension, gettext as _} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

import {HeadsetClient, createProxy} from './lib/client.js';
import {reportFailure} from './lib/failures.js';
import {CompanionIndicator} from './lib/indicator.js';
import {present} from './lib/presentation.js';
import {AppPresetsToggle} from './lib/toggle.js';

export default class ArctisCompanionExtension extends Extension {
    enable() {
        this._cancellable = new Gio.Cancellable();
        this._client = new HeadsetClient(createProxy(Gio.DBus.session));
        this._toggle = new AppPresetsToggle(this._client);
        this._indicator = new CompanionIndicator(this._toggle);
        Main.panel.statusArea.quickSettings.addExternalIndicator(this._indicator);
        this._client.onChange(() => this._render());
        const client = this._client;
        client.start(this._cancellable).then(() => this._renderIfCurrent(client)).catch(reportFailure);
    }

    disable() {
        this._cancellable.cancel();
        this._client.destroy();
        this._indicator.destroyWithItems();
        this._cancellable = null;
        this._client = null;
        this._toggle = null;
        this._indicator = null;
    }

    _renderIfCurrent(client) {
        if (client === this._client)
            this._render();
    }

    _render() {
        const presentation = present(this._client.properties(), _);
        this._toggle.render(presentation);
        this._indicator.render(presentation);
    }
}
