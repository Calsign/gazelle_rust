#!/usr/bin/env bash

set -euo pipefail

cd -- "$(dirname -- "${BASH_SOURCE[0]}")"

# helper to make github actions output pretty
run_group() {
    local name="$1"
    shift

    echo "::group::$name"
    printf "\n\n====== $name ======\n\n\n"

    if "$@"; then
        echo "::endgroup::"
    else
        local status=$?
        echo "::endgroup::"
        return "$status"
    fi
}

run_group "build and test" \
          bazel test --config ci //...

run_group "test gazelle invariance" \
          bazel run //:gazelle -- --mode=diff --strict

cd example

run_group "build and test for ./example directory" \
          bazel test //...

run_group "test gazelle invariance for ./example directory" \
          bazel run //:gazelle -- --mode=diff --strict
