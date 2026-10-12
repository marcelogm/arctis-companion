import Gio from 'gi://Gio';

export function reportFailure(error, warn = console.warn) {
    if (!isCancellation(error))
        warn(`Arctis Companion: ${error.message}`);
}

function isCancellation(error) {
    return error instanceof Gio.IOErrorEnum && error.matches(Gio.IOErrorEnum, Gio.IOErrorEnum.CANCELLED);
}
