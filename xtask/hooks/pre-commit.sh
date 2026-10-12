#!/bin/sh
[ -d xtask ] || exit 0
exec cargo xtask check-rules --staged
