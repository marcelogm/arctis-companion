#!/bin/sh
set -eu
cd "$(dirname "$0")"
fresh=$(mktemp -d)
sh extract.sh "$fresh/fresh.pot"
grep -v POT-Creation-Date arctis-companion-extension.pot > "$fresh/tracked.body"
grep -v POT-Creation-Date "$fresh/fresh.pot" > "$fresh/fresh.body"
diff -u "$fresh/tracked.body" "$fresh/fresh.body"
msgfmt --check --output-file=/dev/null pt_BR.po
msgcmp pt_BR.po arctis-companion-extension.pot
test -z "$(msgattrib --untranslated --no-obsolete pt_BR.po)"
test -z "$(msgattrib --only-fuzzy --no-obsolete pt_BR.po)"
echo "catalogue up to date"
