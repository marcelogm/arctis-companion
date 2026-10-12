const HIDDEN_STATES = ['absent', 'permission-denied'];
const HEADSET_ICON = 'audio-headset-symbolic';
const UNKNOWN_LEVEL_ICON = 'battery-missing-symbolic';
const WARNING_ICONS = new Map([
    ['low', 'battery-caution-symbolic'],
    ['critical', 'battery-level-0-symbolic'],
]);
const PLACEHOLDER = /\{(\w+)\}/g;

export const EMPTY_PROPERTIES = Object.freeze({
    State: 'absent',
    Model: '',
    BatteryLevel: -1,
    Charging: 'unknown',
    BatteryWarning: 'none',
    Route: 'none',
    Presets: [],
    ActivePreset: '',
    ActivePresetReason: ['route', ''],
    AppRuleCount: 0,
    AutomaticPresets: false,
});

export const HIDDEN = Object.freeze({
    visible: false,
    panelIcon: null,
    title: '',
    subtitle: null,
    sensitive: false,
    checked: false,
    header: null,
    items: [],
});

export function present(bag, _ = untranslated) {
    const properties = {...EMPTY_PROPERTIES, ...bag};
    if (bag === null || HIDDEN_STATES.includes(properties.State))
        return HIDDEN;
    return {
        visible: true,
        panelIcon: panelIcon(properties),
        title: _('App Presets'),
        subtitle: subtitle(properties, _),
        sensitive: properties.AppRuleCount > 0,
        checked: properties.AutomaticPresets,
        header: header(properties, _),
        items: items(properties),
    };
}

export function fill(template, values) {
    return template.replace(PLACEHOLDER, (placeholder, name) => values[name] ?? placeholder);
}

export function automaticPresetsAfterClick({sensitive, checked}) {
    return sensitive ? !checked : null;
}

function untranslated(text) {
    return text;
}

function subtitle(properties, _) {
    if (properties.AppRuleCount === 0)
        return _('No app rules');
    const preset = activePresetName(properties);
    if (preset === null)
        return null;
    const [reason, app] = properties.ActivePresetReason;
    return fill(reasonTemplate(reason, app, _), {preset, app});
}

function reasonTemplate(reason, app, _) {
    if (app === '')
        return '{preset}';
    if (reason === 'app')
        return _('{preset} · {app}');
    if (reason === 'manual')
        return _('{preset} · Until {app} stops');
    return '{preset}';
}

function activePresetName({Presets, ActivePreset}) {
    const active = Presets.find(([id]) => id === ActivePreset);
    return active ? active[1] : null;
}

function panelIcon({BatteryWarning, Charging, BatteryLevel}) {
    if (WARNING_ICONS.has(BatteryWarning))
        return WARNING_ICONS.get(BatteryWarning);
    return Charging === 'charging' ? chargingIcon(BatteryLevel) : null;
}

function chargingIcon(level) {
    if (level < 0)
        return UNKNOWN_LEVEL_ICON;
    const tens = Math.min(Math.floor(level / 10) * 10, 90);
    return `battery-level-${tens}-charging-symbolic`;
}

function header(properties, _) {
    return {
        icon: HEADSET_ICON,
        title: properties.Model === '' ? _('Headset') : properties.Model,
        subtitle: batteryText(properties, _),
    };
}

function batteryText({BatteryLevel, Charging}, _) {
    const charging = Charging === 'charging';
    if (BatteryLevel < 0)
        return charging ? _('Charging') : '';
    const percent = BatteryLevel.toLocaleString();
    return fill(charging ? _('Charging · {percent} %') : _('{percent} %'), {percent});
}

function items(properties) {
    const sensitive = canPick(properties);
    return properties.Presets.map(([id, label]) => ({
        id,
        label,
        checked: id === properties.ActivePreset,
        sensitive,
    }));
}

function canPick({State, Route}) {
    return State !== 'connecting' && !(State === 'awaiting-headset' && Route !== 'bluetooth');
}
