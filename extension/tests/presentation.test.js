import {test} from './support/harness.js';
import {assertEqual} from './support/assert.js';
import {connectedBag} from './support/bags.js';
import {
    HIDDEN,
    automaticPresetsAfterClick,
    fill,
    present,
} from '../arctis-companion@marcelogm.github.io/lib/presentation.js';

const VISIBLE_STATES = ['bluetooth-only', 'connecting', 'awaiting-headset', 'headset-off', 'connected'];

function presentWith(overrides) {
    return present(connectedBag(overrides));
}

test('hidden when the app has no bus name owner', () => assertEqual(present(null), HIDDEN));

test('hidden while no dongle is found', () => assertEqual(presentWith({State: 'absent'}), HIDDEN));

test('hidden while the dongle is denied', () => assertEqual(presentWith({State: 'permission-denied'}), HIDDEN));

test('visible in every other state', () =>
    assertEqual(VISIBLE_STATES.map(State => presentWith({State}).visible), VISIBLE_STATES.map(() => true)));

test('missing properties fall back to empty values', () => {
    const presentation = present({State: 'connected'});
    assertEqual([presentation.checked, presentation.sensitive], [false, false]);
});

test('fill replaces named placeholders and keeps unknown ones', () =>
    assertEqual(fill('{preset} · {app} {other}', {preset: 'Flat', app: 'Firefox'}), 'Flat · Firefox {other}'));

test('toggle is titled App Presets', () => assertEqual(present(connectedBag()).title, 'App Presets'));

test('toggle is checked when automatic presets are on', () =>
    assertEqual([true, false].map(on => presentWith({AutomaticPresets: on}).checked), [true, false]));

test('toggle is insensitive without app rules', () =>
    assertEqual([0, 1].map(count => presentWith({AppRuleCount: count}).sensitive), [false, true]));

test('a click asks for the opposite of checked', () =>
    assertEqual([true, false].map(on => automaticPresetsAfterClick(presentWith({AutomaticPresets: on}))),
        [false, true]));

test('a click asks for nothing while insensitive', () =>
    assertEqual([presentWith({AppRuleCount: 0}), HIDDEN].map(automaticPresetsAfterClick), [null, null]));

test('subtitle reads No app rules without app rules', () =>
    assertEqual(presentWith({AppRuleCount: 0, ActivePresetReason: ['app', 'Firefox']}).subtitle, 'No app rules'));

test('subtitle ignores the app for other reasons', () =>
    assertEqual(presentWith({ActivePresetReason: ['route', 'Firefox']}).subtitle, 'Flat'));

test('subtitle names the route preset', () => assertEqual(presentWith({}).subtitle, 'Flat'));

test('subtitle names the app whose rule chose the preset', () =>
    assertEqual(presentWith({ActivePreset: 'footsteps', ActivePresetReason: ['app', 'Firefox']}).subtitle,
        'Footsteps · Firefox'));

test('subtitle says until when a manual pick ends', () =>
    assertEqual(presentWith({ActivePreset: 'bass-boost', ActivePresetReason: ['manual', 'Firefox']}).subtitle,
        'Bass Boost · Until Firefox stops'));

test('subtitle of a manual pick without ruled apps is the preset', () =>
    assertEqual(presentWith({ActivePreset: 'bass-boost', ActivePresetReason: ['manual', '']}).subtitle,
        'Bass Boost'));

test('subtitle is empty when the active preset is unknown', () =>
    assertEqual(presentWith({ActivePreset: 'gone'}).subtitle, null));

test('texts go through the translator before placeholders are filled', () => {
    const presentation = present(connectedBag({ActivePreset: 'footsteps', ActivePresetReason: ['app', 'Firefox']}),
        text => `<${text}>`);
    assertEqual([presentation.title, presentation.subtitle], ['<App Presets>', '<Footsteps · Firefox>']);
});

test('no panel icon without a warning or charging', () => assertEqual(presentWith({}).panelIcon, null));

test('panel icon shows a low battery warning', () =>
    assertEqual(presentWith({BatteryWarning: 'low', BatteryLevel: 14}).panelIcon, 'battery-caution-symbolic'));

test('panel icon shows a critical battery warning', () =>
    assertEqual(presentWith({BatteryWarning: 'critical', BatteryLevel: 4}).panelIcon, 'battery-level-0-symbolic'));

test('panel icon shows the charging level in tens', () =>
    assertEqual([0, 47, 100].map(BatteryLevel => presentWith({Charging: 'charging', BatteryLevel}).panelIcon), [
        'battery-level-0-charging-symbolic',
        'battery-level-40-charging-symbolic',
        'battery-level-90-charging-symbolic',
    ]));

test('panel icon of charging with an unknown level', () =>
    assertEqual(presentWith({Charging: 'charging', BatteryLevel: -1}).panelIcon, 'battery-missing-symbolic'));

test('a warning wins over charging', () =>
    assertEqual(presentWith({Charging: 'charging', BatteryWarning: 'low'}).panelIcon, 'battery-caution-symbolic'));

test('no panel icon once charged', () => assertEqual(presentWith({Charging: 'charged'}).panelIcon, null));

test('header shows the model and the battery level', () =>
    assertEqual(presentWith({}).header, {icon: 'audio-headset-symbolic', title: 'Arctis Nova 5', subtitle: '73 %'}));

test('header shows charging with the level', () =>
    assertEqual(presentWith({Charging: 'charging'}).header.subtitle, 'Charging · 73 %'));

test('header hides an unknown level', () =>
    assertEqual(['discharging', 'charging'].map(Charging => presentWith({Charging, BatteryLevel: -1}).header.subtitle),
        ['', 'Charging']));

test('header falls back to Headset without a model', () =>
    assertEqual(presentWith({State: 'bluetooth-only', Model: ''}).header.title, 'Headset'));

test('items list every preset and check the active one', () =>
    assertEqual(presentWith({ActivePreset: 'footsteps'}).items.map(({id, label, checked}) => [id, label, checked]), [
        ['flat', 'Flat', false],
        ['footsteps', 'Footsteps', true],
        ['bass-boost', 'Bass Boost', false],
    ]));

test('items are insensitive while connecting', () =>
    assertEqual(presentWith({State: 'connecting'}).items.map(item => item.sensitive), [false, false, false]));

test('items are insensitive while awaiting the headset on the dongle', () =>
    assertEqual(presentWith({State: 'awaiting-headset', Route: 'dongle'}).items[0].sensitive, false));

test('items stay sensitive while awaiting the headset on Bluetooth', () =>
    assertEqual(presentWith({State: 'awaiting-headset', Route: 'bluetooth'}).items[0].sensitive, true));

test('items stay sensitive while the headset is off', () =>
    assertEqual(presentWith({State: 'headset-off'}).items[0].sensitive, true));
