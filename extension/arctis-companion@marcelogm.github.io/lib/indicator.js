import GObject from 'gi://GObject';

import {SystemIndicator} from 'resource:///org/gnome/shell/ui/quickSettings.js';

export const CompanionIndicator = GObject.registerClass(
class CompanionIndicator extends SystemIndicator {
    _init(toggle) {
        super._init();
        this._icon = this._addIndicator();
        this._icon.visible = false;
        this.quickSettingsItems.push(toggle);
    }

    render({panelIcon}) {
        if (panelIcon !== null)
            this._icon.icon_name = panelIcon;
        this._icon.visible = panelIcon !== null;
    }

    destroyWithItems() {
        this.quickSettingsItems.forEach(item => item.destroy());
        this.destroy();
    }
});
