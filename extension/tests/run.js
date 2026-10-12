import GLib from 'gi://GLib';
import System from 'system';

import {runAll} from './support/harness.js';
import './assert.test.js';
import './metadata.test.js';
import './presentation.test.js';
import './failures.test.js';
import './client.test.js';

const loop = new GLib.MainLoop(null, false);
let failures = 1;
runAll()
    .then(count => {
        failures = count;
    })
    .catch(logError)
    .finally(() => loop.quit());
loop.run();
System.exit(failures === 0 ? 0 : 1);
