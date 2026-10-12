#!/bin/sh
set -eu
here=$(cd "$(dirname "$0")" && pwd)
root=$(mktemp -d)
export XDG_RUNTIME_DIR="$root/run"
export PIPEWIRE_RUNTIME_DIR="$root/run"
export XDG_CONFIG_HOME="$root/config"
export XDG_STATE_HOME="$root/state"
export XDG_DATA_HOME="$root/data"
export XDG_CACHE_HOME="$root/cache"
export ARCTIS_HEADLESS_PIPEWIRE=1
unset PIPEWIRE_REMOTE PULSE_SERVER
mkdir -p -m 700 "$XDG_RUNTIME_DIR"
mkdir -p "$XDG_CONFIG_HOME/pipewire/pipewire.conf.d" "$XDG_CONFIG_HOME/wireplumber/wireplumber.conf.d"
cp "$here/pipewire.conf" "$XDG_CONFIG_HOME/pipewire/pipewire.conf.d/50-arctis-headless.conf"
cp "$here/wireplumber.conf" "$XDG_CONFIG_HOME/wireplumber/wireplumber.conf.d/50-arctis-headless.conf"
status=0
dbus-run-session -- "$here/session.sh" "$@" || status=$?
rm -rf "$root"
exit "$status"
