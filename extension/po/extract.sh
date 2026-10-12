#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
xgettext --from-code=UTF-8 --language=JavaScript --keyword=_ --add-location=file --sort-by-file \
    --package-name=arctis-companion-extension \
    --output="${1:-po/arctis-companion-extension.pot}" \
    arctis-companion@marcelogm.github.io/extension.js arctis-companion@marcelogm.github.io/lib/*.js
