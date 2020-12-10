#!/bin/bash

##
#  Bloom
#
#  HTTP REST API caching middleware
#  Copyright: 2020, Valerian Saliou <valerian@valeriansaliou.name>
#  License: Mozilla Public License v2.0 (MPL v2.0)
##

# Read arguments
while [ "$1" != "" ]; do
    argument_key=`echo $1 | awk -F= '{print $1}'`
    argument_value=`echo $1 | awk -F= '{print $2}'`

    case $argument_key in
        -v | --version)
            BLOOM_VERSION="$argument_value"
            ;;
        *)
            echo "Unknown argument received: '$argument_key'"
            exit 1
            ;;
    esac

    shift
done

# Ensure release version is provided
if [ -z "$BLOOM_VERSION" ]; then
  echo "No Bloom release version was provided, please provide it using '--version'"

  exit 1
fi

# Define build pipeline
function build_for_target {
    OS="$2" DIST="$3" ARCH="$1" VERSION="$BLOOM_VERSION" ./packpack/packpack
    release_result=$?

    if [ $release_result -eq 0 ]; then
        mv ./build/*$4 ./

        echo "Result: Packaged architecture: $1 for OS: $2:$3 (*$4)"
    fi

    return $release_result
}

# Run release tasks
ABSPATH=$(cd "$(dirname "$0")"; pwd)
BASE_DIR="$ABSPATH/../"

rc=0

pushd "$BASE_DIR" > /dev/null
    echo "Executing packages build steps for Bloom v$BLOOM_VERSION..."

    # Initialize `packpack`
    rm -rf ./packpack && \
        git clone https://github.com/packpack/packpack.git packpack
    rc=$?

    # Proceed build for each target?
    if [ $rc -eq 0 ]; then
        build_for_target "x86_64" "debian" "buster" ".deb"
        rc=$?
    fi

    # Cleanup environment
    rm -rf ./build ./packpack

    if [ $rc -eq 0 ]; then
        echo "Success: Done executing packages build steps for Bloom v$BLOOM_VERSION"
    else
        echo "Error: Failed executing packages build steps for Bloom v$BLOOM_VERSION"
    fi
popd > /dev/null

exit $rc
