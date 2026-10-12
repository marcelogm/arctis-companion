import GLib from 'gi://GLib';

const POLL_INTERVAL_MS = 10;

export function delay(milliseconds) {
    return new Promise(resolve => {
        GLib.timeout_add(GLib.PRIORITY_DEFAULT, milliseconds, () => {
            resolve();
            return GLib.SOURCE_REMOVE;
        });
    });
}

export async function waitFor(condition, timeoutMs = 2000) {
    const deadline = GLib.get_monotonic_time() + timeoutMs * 1000;
    while (!condition()) {
        if (GLib.get_monotonic_time() > deadline)
            throw new Error('timed out waiting for a condition');
        await delay(POLL_INTERVAL_MS);
    }
}
