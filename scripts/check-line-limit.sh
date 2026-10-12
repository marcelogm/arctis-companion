#!/bin/sh
limit=400
exempt='(^|/)(Cargo\.lock|cargo-sources\.json)$|\.(po|pot|svg|png)$'

if [ "$1" = "--staged" ]; then
  files=$(git diff --cached --name-only --diff-filter=ACMR)
else
  files=$(git ls-files)
fi

status=0
for file in $files; do
  echo "$file" | grep -qE "$exempt" && continue
  [ -f "$file" ] || continue
  lines=$(wc -l < "$file")
  if [ "$lines" -gt "$limit" ]; then
    echo "line-limit: $file has $lines lines (max $limit), refactor it" >&2
    status=1
  fi
done
exit $status
