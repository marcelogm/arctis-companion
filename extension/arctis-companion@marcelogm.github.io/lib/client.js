import Gio from 'gi://Gio';
import GLib from 'gi://GLib';

export const BUS_NAME = 'io.github.marcelogm.ArctisCompanion';
export const DESKTOP_ID = `${BUS_NAME}.desktop`;
export const OBJECT_PATH = '/io/github/marcelogm/ArctisCompanion';
export const INTERFACE_NAME = 'io.github.marcelogm.ArctisCompanion.Headset1';

const INTERFACE_FILE = '../dbus/io.github.marcelogm.ArctisCompanion.Headset1.xml';
const CALL_FLAGS = Gio.DBusCallFlags.NO_AUTO_START;
const DEFAULT_TIMEOUT = -1;

Gio._promisify(Gio.DBusProxy.prototype, 'call');
Gio._promisify(Gio.DBusConnection.prototype, 'call');

export function interfaceXml() {
    const uri = GLib.Uri.resolve_relative(import.meta.url, INTERFACE_FILE, GLib.UriFlags.NONE);
    const [path] = GLib.filename_from_uri(uri);
    const [, bytes] = GLib.file_get_contents(path);
    return new TextDecoder().decode(bytes);
}

export function interfaceInfo() {
    return Gio.DBusNodeInfo.new_for_xml(interfaceXml()).lookup_interface(INTERFACE_NAME);
}

export function createProxy(connection) {
    return new Gio.DBusProxy({
        g_connection: connection,
        g_name: BUS_NAME,
        g_object_path: OBJECT_PATH,
        g_interface_name: INTERFACE_NAME,
        g_interface_info: interfaceInfo(),
        g_flags: Gio.DBusProxyFlags.DO_NOT_AUTO_START,
    });
}

export function platformData(activationToken) {
    if (activationToken === null)
        return {};
    const token = GLib.Variant.new_string(activationToken);
    return {'activation-token': token, 'desktop-startup-id': token};
}

export class HeadsetClient {
    #proxy;
    #handlerIds = [];

    constructor(proxy) {
        this.#proxy = proxy;
    }

    start(cancellable = null) {
        return this.#proxy.init_async(GLib.PRIORITY_DEFAULT, cancellable);
    }

    properties() {
        if (this.#proxy.get_name_owner() === null)
            return null;
        const names = this.#proxy.get_cached_property_names() ?? [];
        return Object.fromEntries(names.map(name => [name, this.#proxy.get_cached_property(name).deepUnpack()]));
    }

    onChange(callback) {
        this.#handlerIds.push(
            this.#proxy.connect('g-properties-changed', () => callback()),
            this.#proxy.connect('notify::g-name-owner', () => callback()));
    }

    setAutomaticPresets(enabled) {
        const parameters = new GLib.Variant('(ssv)',
            [INTERFACE_NAME, 'AutomaticPresets', GLib.Variant.new_boolean(enabled)]);
        return this.#proxy.call('org.freedesktop.DBus.Properties.Set', parameters, CALL_FLAGS, DEFAULT_TIMEOUT, null);
    }

    activatePreset(id) {
        return this.#proxy.call('ActivatePreset', new GLib.Variant('(s)', [id]), CALL_FLAGS, DEFAULT_TIMEOUT, null);
    }

    activateApplication(activationToken = null) {
        const parameters = new GLib.Variant('(a{sv})', [platformData(activationToken)]);
        return this.#proxy.get_connection().call(BUS_NAME, OBJECT_PATH, 'org.freedesktop.Application', 'Activate',
            parameters, null, CALL_FLAGS, DEFAULT_TIMEOUT, null);
    }

    destroy() {
        for (const id of this.#handlerIds.splice(0))
            this.#proxy.disconnect(id);
    }
}
