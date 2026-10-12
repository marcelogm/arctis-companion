import GLib from 'gi://GLib';

import {connectedBag} from './support/bags.js';
import {FakeHeadsetService} from './support/fake_headset_service.js';

const SCENARIOS = {
    'connected': {},
    'low-battery': {BatteryLevel: 14, BatteryWarning: 'low'},
    'charging': {Charging: 'charging', BatteryLevel: 47},
    'app-rule': {ActivePreset: 'footsteps', ActivePresetReason: ['app', 'Firefox']},
    'manual-pick': {ActivePreset: 'bass-boost', ActivePresetReason: ['manual', 'Firefox']},
    'no-rules': {AppRuleCount: 0},
    'connecting': {State: 'connecting'},
    'absent': {State: 'absent'},
};

const [scenario = 'connected'] = ARGV;
const loop = new GLib.MainLoop(null, false);
FakeHeadsetService.start(connectedBag(SCENARIOS[scenario]))
    .then(() => print(`fake app serving scenario ${scenario}`))
    .catch(error => {
        logError(error);
        loop.quit();
    });
loop.run();
