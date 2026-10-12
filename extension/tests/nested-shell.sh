#!/bin/sh
set -eu
mode=${1:-smoke}
scenario=${2:-connected}
extension_root=$(cd "$(dirname "$0")/.." && pwd)
session=$(mktemp -d)
export XDG_DATA_HOME="$session/data" XDG_CONFIG_HOME="$session/config" XDG_CACHE_HOME="$session/cache"
export GSETTINGS_BACKEND=keyfile GIO_USE_VFS=local
unset DBUS_SESSION_BUS_ADDRESS
if [ "$mode" != devkit ]; then
    export XDG_RUNTIME_DIR="$session/runtime"
    mkdir -m 700 "$XDG_RUNTIME_DIR"
    unset WAYLAND_DISPLAY DISPLAY
fi
cd "$extension_root"
npm run --silent pack
gnome-extensions install --force dist/arctis-companion@marcelogm.github.io.shell-extension.zip
exec dbus-run-session -- sh tests/shell-session.sh "$mode" "$scenario" "$session/shell.log"
