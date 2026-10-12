import GLib from 'gi://GLib';

export const UUID = 'arctis-companion@marcelogm.github.io';

const [supportFile] = GLib.filename_from_uri(import.meta.url);
const EXTENSION_ROOT = ancestor(supportFile, 3);

export function extensionPath(...parts) {
    return GLib.build_filenamev([EXTENSION_ROOT, UUID, ...parts]);
}

export function readText(path) {
    const [, bytes] = GLib.file_get_contents(path);
    return new TextDecoder().decode(bytes);
}

function ancestor(path, levels) {
    return levels === 0 ? path : ancestor(GLib.path_get_dirname(path), levels - 1);
}
