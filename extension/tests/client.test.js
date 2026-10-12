import Gio from 'gi://Gio';
import GLib from 'gi://GLib';

import {test} from './support/harness.js';
import {assertEqual, assertRejects, assertTrue} from './support/assert.js';
import {armActivationMarker, startAppByName, wasActivated} from './support/activation.js';
import {connectedBag} from './support/bags.js';
import {FakeHeadsetService} from './support/fake_headset_service.js';
import {delay, waitFor} from './support/loop.js';
import {extensionPath, readText} from './support/paths.js';
import {HeadsetClient, createProxy, interfaceInfo} from '../arctis-companion@marcelogm.github.io/lib/client.js';

const HEADSET1_PROPERTIES = [
    'State', 'Model', 'BatteryLevel', 'Charging', 'BatteryWarning', 'Route', 'Presets', 'ActivePreset',
    'ActivePresetReason', 'AppRuleCount', 'AutomaticPresets',
];

async function startClient() {
    const client = new HeadsetClient(createProxy(Gio.DBus.session));
    await client.start();
    return client;
}

async function withRunningApp(body) {
    const service = await FakeHeadsetService.start();
    const client = await startClient();
    try {
        await body(client, service);
    } finally {
        client.destroy();
        await service.stop();
    }
}

test('client imports nothing from the Shell', () =>
    assertTrue(!readText(extensionPath('lib', 'client.js')).includes('resource:///'),
        'client.js must not import Shell modules'));

test('interface copy declares every Headset1 property', () =>
    assertEqual(interfaceInfo().properties.map(property => property.name), HEADSET1_PROPERTIES));

test('proxy never starts the app', () => {
    const flags = createProxy(Gio.DBus.session).g_flags;
    assertTrue((flags & Gio.DBusProxyFlags.DO_NOT_AUTO_START) !== 0, 'DO_NOT_AUTO_START must be set');
});

test('no properties while the app is absent', async () => {
    const client = await startClient();
    assertEqual(client.properties(), null);
    client.destroy();
});

test('client follows the app appearing and vanishing', async () => {
    const client = await startClient();
    let changes = 0;
    client.onChange(() => {
        changes += 1;
    });
    const service = await FakeHeadsetService.start();
    await waitFor(() => client.properties() !== null);
    assertEqual(client.properties(), connectedBag());
    await service.stop();
    await waitFor(() => client.properties() === null);
    assertTrue(changes >= 2, `expected two change notifications, got ${changes}`);
    client.destroy();
});

test('client relays changed properties', () => withRunningApp(async (client, service) => {
    let changes = 0;
    client.onChange(() => {
        changes += 1;
    });
    service.update({BatteryLevel: 14, BatteryWarning: 'low'});
    await waitFor(() => client.properties().BatteryLevel === 14);
    assertEqual([client.properties().BatteryWarning, changes > 0], ['low', true]);
}));

test('destroy stops change notifications', () => withRunningApp(async (client, service) => {
    let changes = 0;
    client.onChange(() => {
        changes += 1;
    });
    client.destroy();
    service.update({BatteryLevel: 50});
    await delay(100);
    assertEqual(changes, 0);
}));

test('toggling writes AutomaticPresets', () => withRunningApp(async (client, service) => {
    await client.setAutomaticPresets(false);
    await waitFor(() => client.properties().AutomaticPresets === false);
    assertEqual(service.automaticPresetWrites, [false]);
}));

test('picking a preset calls ActivatePreset', () => withRunningApp(async (client, service) => {
    await client.activatePreset('bass-boost');
    assertEqual(service.activatedPresets, ['bass-boost']);
}));

test('ActivatePreset errors reach the caller', () => withRunningApp(async (client, service) => {
    service.update({Route: 'none'});
    await assertRejects(client.activatePreset('flat'), /Error\.NoRoute/);
}));

test('Headset Settings activates the app', () => withRunningApp(async (client, service) => {
    await client.activateApplication();
    assertEqual(service.activations, 1);
}));

test('neither the proxy nor its calls start the app', async () => {
    const marker = await armActivationMarker();
    const client = await startClient();
    await Promise.allSettled([
        client.setAutomaticPresets(true),
        client.activatePreset('flat'),
        client.activateApplication(),
    ]);
    await delay(200);
    client.destroy();
    assertTrue(!wasActivated(marker), 'the app was started');
});

test('the test bus can start the app when asked', async () => {
    const marker = await armActivationMarker();
    await startAppByName().catch(() => null);
    await waitFor(() => wasActivated(marker));
    GLib.unlink(marker);
});
