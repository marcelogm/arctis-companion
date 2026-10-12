import Gio from 'gi://Gio';
import GLib from 'gi://GLib';

import {BUS_NAME} from '../../arctis-companion@marcelogm.github.io/lib/client.js';

export async function armActivationMarker() {
    const marker = GLib.build_filenamev([GLib.get_tmp_dir(), `arctis-activation-${GLib.random_int()}`]);
    await busCall('UpdateActivationEnvironment', new GLib.Variant('(a{ss})', [{ARCTIS_ACTIVATION_MARKER: marker}]));
    return marker;
}

export function wasActivated(marker) {
    return GLib.file_test(marker, GLib.FileTest.EXISTS);
}

export function startAppByName() {
    return busCall('StartServiceByName', new GLib.Variant('(su)', [BUS_NAME, 0]));
}

function busCall(method, parameters) {
    return Gio.DBus.session.call('org.freedesktop.DBus', '/org/freedesktop/DBus', 'org.freedesktop.DBus', method,
        parameters, null, Gio.DBusCallFlags.NONE, -1, null);
}
