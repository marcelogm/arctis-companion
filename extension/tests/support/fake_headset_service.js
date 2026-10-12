import Gio from 'gi://Gio';
import GLib from 'gi://GLib';

import {BUS_NAME, OBJECT_PATH, interfaceInfo, interfaceXml} from '../../arctis-companion@marcelogm.github.io/lib/client.js';
import {connectedBag} from './bags.js';
import {waitFor} from './loop.js';

const NO_ROUTE = 'io.github.marcelogm.ArctisCompanion.Error.NoRoute';
const APPLICATION_XML = `<node>
  <interface name="org.freedesktop.Application">
    <method name="Activate">
      <arg name="platform_data" type="a{sv}" direction="in"/>
    </method>
  </interface>
</node>`;
const CONNECTION_FLAGS = Gio.DBusConnectionFlags.AUTHENTICATION_CLIENT |
    Gio.DBusConnectionFlags.MESSAGE_BUS_CONNECTION;

Gio._promisify(Gio.DBusConnection, 'new_for_address');
Gio._promisify(Gio.DBusConnection.prototype, 'close');

export class FakeHeadsetService {
    automaticPresetWrites = [];
    activatedPresets = [];
    activations = 0;
    activationTokens = [];
    #bag;
    #connection = null;
    #headset = null;
    #application = null;
    #ownerId = 0;
    #owned = false;

    static async start(bag = connectedBag()) {
        const service = new FakeHeadsetService(bag);
        await service.#open();
        return service;
    }

    constructor(bag) {
        this.#bag = {...bag};
    }

    update(changes) {
        Object.assign(this.#bag, changes);
        for (const [name, value] of Object.entries(changes))
            this.#headset.emit_property_changed(name, new GLib.Variant(signatureOf(name), value));
        this.#headset.flush();
    }

    async stop() {
        if (this.#connection.is_closed())
            return;
        Gio.bus_unown_name(this.#ownerId);
        this.#headset.unexport();
        this.#application.unexport();
        await this.#connection.close(null);
    }

    async #open() {
        const address = Gio.dbus_address_get_for_bus_sync(Gio.BusType.SESSION, null);
        this.#connection = await Gio.DBusConnection.new_for_address(address, CONNECTION_FLAGS, null, null);
        this.#headset = Gio.DBusExportedObject.wrapJSObject(interfaceXml(), this.#headsetImplementation());
        this.#headset.export(this.#connection, OBJECT_PATH);
        this.#application = Gio.DBusExportedObject.wrapJSObject(APPLICATION_XML, {Activate: platformData => this.#activate(platformData)});
        this.#application.export(this.#connection, OBJECT_PATH);
        this.#ownerId = Gio.bus_own_name_on_connection(this.#connection, BUS_NAME, Gio.BusNameOwnerFlags.NONE,
            () => this.#acquired(), null);
        await waitFor(() => this.#owned);
    }

    #headsetImplementation() {
        return new Proxy({}, {
            get: (target, member) => this.#member(member),
            set: (target, member, value) => this.#write(member, value),
        });
    }

    #member(member) {
        if (member === 'ActivatePresetAsync')
            return (parameters, invocation) => this.#activatePreset(parameters, invocation);
        return this.#bag[member];
    }

    #write(member, value) {
        if (member === 'AutomaticPresets') {
            this.automaticPresetWrites.push(value);
            this.update({AutomaticPresets: value});
        }
        return true;
    }

    #activatePreset([id], invocation) {
        if (this.#bag.Route === 'none') {
            invocation.return_dbus_error(NO_ROUTE, 'No picked path');
            return;
        }
        this.activatedPresets.push(id);
        invocation.return_value(null);
    }

    #activate(platformData) {
        this.activations += 1;
        this.activationTokens.push(platformData['activation-token']?.unpack() ?? null);
    }

    #acquired() {
        this.#owned = true;
    }
}

function signatureOf(name) {
    return interfaceInfo().lookup_property(name).signature;
}
