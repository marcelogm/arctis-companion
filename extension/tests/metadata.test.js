import GLib from 'gi://GLib';

import {test} from './support/harness.js';
import {assertEqual, assertTrue} from './support/assert.js';
import {UUID, extensionPath, readText} from './support/paths.js';

function metadata() {
    return JSON.parse(readText(extensionPath('metadata.json')));
}

test('metadata targets GNOME Shell 50 only', () => assertEqual(metadata()['shell-version'], ['50']));

test('metadata uuid names the extension directory', () => assertEqual(metadata().uuid, UUID));

test('metadata names the extension gettext domain', () =>
    assertEqual(metadata()['gettext-domain'], 'arctis-companion-extension'));

test('extension ships no preferences window', () =>
    assertTrue(!GLib.file_test(extensionPath('prefs.js'), GLib.FileTest.EXISTS), 'prefs.js must not exist'));
