#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
output="${1:-po/arctis-companion.pot}"
lists=$(mktemp -d)
common="--from-code=UTF-8 --force-po --add-location=file --sort-by-file --package-name=arctis-companion"
find crates -path '*/src/*' -name '*.rs' -not -path '*/testkit/*' | LC_ALL=C sort > "$lists/rust"
find crates/ui/resources -name '*.blp' | LC_ALL=C sort > "$lists/blueprint"
xgettext $common --language=Rust --keyword=msgid --keyword=forms:1,2 \
    --files-from="$lists/rust" --output="$output"
xgettext $common --join-existing --language=C --keyword=_ --keyword=C_:1c,2 \
    --files-from="$lists/blueprint" --output="$output"
xgettext $common --join-existing --language=Desktop \
    --output="$output" data/io.github.marcelogm.ArctisCompanion.desktop.in
xgettext $common --join-existing \
    --output="$output" data/io.github.marcelogm.ArctisCompanion.metainfo.xml.in
rm -r "$lists"
