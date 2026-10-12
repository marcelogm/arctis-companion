#!/bin/sh
[ -d xtask ] || exit 0
exec cargo xtask check-commits --message "$1"
