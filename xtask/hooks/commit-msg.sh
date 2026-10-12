#!/bin/sh
if [ -d xtask ]; then
    exec cargo xtask check-commits --message "$1"
fi
if [ -f .githooks/commit-msg ]; then
    exec sh .githooks/commit-msg "$1"
fi
