#!/bin/sh
set -eu
cargo test --package arctis-audio --test unreachable -- --ignored
cargo test --package arctis-audio --test reconnect -- --ignored
pipewire > "$XDG_RUNTIME_DIR/pipewire.log" 2>&1 &
pipewire_pid=$!
wireplumber > "$XDG_RUNTIME_DIR/wireplumber.log" 2>&1 &
wireplumber_pid=$!
trap 'kill "$wireplumber_pid" "$pipewire_pid" 2>/dev/null || true' EXIT
timeout 15 sh -c 'until wpctl inspect @DEFAULT_AUDIO_SINK@ > /dev/null 2>&1; do sleep 0.2; done'
cargo test --package arctis-audio --test headless -- --ignored --test-threads=1 "$@"
