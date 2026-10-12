#!/bin/sh
set -eu
mode=$1
scenario=$2
log=$3
uuid=arctis-companion@marcelogm.github.io

run_headless() {
    gnome-shell --headless --wayland --no-x11 --virtual-monitor 1280x800 --wayland-display=arctis-smoke \
        > "$log" 2>&1 &
    shell=$!
    timeout 30 sh -c "until gnome-extensions info $uuid | grep -q 'State: ACTIVE'; do sleep 0.5; done"
    sleep 2
    gnome-extensions disable "$uuid"
    timeout 10 sh -c "until gnome-extensions info $uuid | grep -q 'State: INACTIVE'; do sleep 0.5; done"
    kill "$shell"
}

run_devkit() {
    gnome-shell --devkit --wayland > "$log" 2>&1
}

gsettings set org.gnome.shell disable-user-extensions false
gsettings set org.gnome.shell enabled-extensions "['$uuid']"
gjs -m tests/fake_app.js "$scenario" &
fake_app=$!
trap 'kill "$fake_app"' EXIT
if [ "$mode" = devkit ]; then run_devkit; else run_headless; fi
if grep -n "JS ERROR\|JS WARNING\|Arctis Companion:" "$log"; then
    exit 1
fi
echo "nested shell passed: $mode $scenario"
