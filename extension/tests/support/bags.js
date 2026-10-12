export function connectedBag(overrides = {}) {
    return {
        State: 'connected',
        Model: 'Arctis Nova 5',
        BatteryLevel: 73,
        Charging: 'discharging',
        BatteryWarning: 'none',
        Route: 'dongle',
        Presets: [['flat', 'Flat'], ['footsteps', 'Footsteps'], ['bass-boost', 'Bass Boost']],
        ActivePreset: 'flat',
        ActivePresetReason: ['route', ''],
        AppRuleCount: 2,
        AutomaticPresets: true,
        ...overrides,
    };
}
