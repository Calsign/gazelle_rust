#!/bin/sh
OUT=$($TEST_SRCDIR/go_sdk/bin/gofmt -d -e -l "$@" 2>&1)
echo "$OUT"
test -z "$OUT"
