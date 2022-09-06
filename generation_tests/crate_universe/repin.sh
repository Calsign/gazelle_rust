#!/bin/bash
set -euo pipefail

# NOTE: Run this script through bazel, see README.md.

CRATE_UNIVERSE_DIR="$BUILD_WORKSPACE_DIRECTORY/generation_tests/crate_universe"

# use a persistent directory for --output_user_root so that we don't have to refetch each time
# we re-run the script
OUTPUT_USER_ROOT="/tmp/gazelle_rust_repin_output_user_root"

for dir in $CRATE_UNIVERSE_DIR/*/ ; do
    name=$(basename $dir)

    # if a particular directory is specified, only re-pin that one
    if [ -n "${1+x}" ] && [ "$1" != "$name" ]; then
        continue
    fi

    echo ""
    echo "Repinning $name"

    # NOTE: if we use mktemp to get a new directory, we don't hit cache since the directory name is
    # different.
    tmpdir="/tmp/gazelle_rust_repin_${name}"

    # delete old one if it still exists
    rm -rf $tmpdir

    echo "Copying to temporary directory $tmpdir"
    cp -r $dir/. $tmpdir

    pushd . > /dev/null
    cd $tmpdir

    echo "Renaming build files"
    # turn BUILD.in files into real BUILD.bazel files
    find . -type f -name "BUILD.in" -exec sh -c "f={}; mv \$f \${f%.in}.bazel" \;
    # edit to the WORKSPACE to make the gazelle_rust new_local_repository point at the correct path
    sed -i 's|../../..|'$BUILD_WORKSPACE_DIRECTORY'|g' WORKSPACE

    if [[ $name == vendored* ]]; then
        echo "Repinning vendored crates"
        bazel --output_user_root=$OUTPUT_USER_ROOT run //3rdparty:crates_vendor -- --repin

        echo "Renaming build files back"
        find . -type f -name "BUILD.bazel" -exec sh -c "f={}; mv \$f \${f%.bazel}.in" \;

        echo "Copying back into repo"
        cp Cargo.lock $dir
        # copy the entire 3rdparty directory; delete it first so that removed packages are reflected
        # properly
        rm -r $dir/3rdparty
        cp -r 3rdparty/. $dir/3rdparty
    else
        echo "Syncing crates"
        CARGO_BAZEL_REPIN=true bazel --output_user_root=$OUTPUT_USER_ROOT sync --only=crates

        echo "Copying back into repo"
        # we only need to copy the two lockfiles
        cp Cargo.Bazel.lock Cargo.lock $dir
    fi

    popd > /dev/null

    rm -r $tmpdir
done
