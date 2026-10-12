import Gio from 'gi://Gio';
import GLib from 'gi://GLib';

import {test} from './support/harness.js';
import {assertEqual} from './support/assert.js';
import {reportFailure} from '../arctis-companion@marcelogm.github.io/lib/failures.js';

function warningsFor(error) {
    const warnings = [];
    reportFailure(error, message => warnings.push(message));
    return warnings;
}

test('a cancelled call is not reported', () =>
    assertEqual(warningsFor(new GLib.Error(Gio.IOErrorEnum, Gio.IOErrorEnum.CANCELLED, 'cancelled')), []));

test('a D-Bus error is reported with its message', () =>
    assertEqual(warningsFor(new GLib.Error(Gio.DBusError, Gio.DBusError.SERVICE_UNKNOWN, 'gone')),
        ['Arctis Companion: gone']));

test('a JavaScript error is reported with its message', () =>
    assertEqual(warningsFor(new Error('boom')), ['Arctis Companion: boom']));
