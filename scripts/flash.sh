#!/usr/bin/env bash
# shellcheck disable=SC2155
export BUILD_TIME=$(date +'%Y-%m-%d %H:%M:%S')
echo "current use version(time): $BUILD_TIME"

set -e

BUILD_MODE=""
case "$1" in
"" | "release")
    cargo build --release
    printenv BUILD_TIME
    rm ./asset/upgrade_file/*
    espflash save-image --chip esp32s3 target/xtensa-esp32s3-espidf/release/ele_ds_client_rust "./asset/upgrade_file/${BUILD_TIME}.bin"
    cp ./asset/upgrade_file/* ../general_serve/asset/server_root_path/upgrade/ele_ds_client_rust/

    echo "release successes"
    BUILD_MODE="release"
    ;;
"debug")
    bash scripts/build.sh debug
    BUILD_MODE="debug"
    ;;
*)
    echo "Wrong argument. Only \"debug\"/\"release\" arguments are supported"
    exit 1
    ;;
esac
unset BUILD_TIME
web-flash --chip esp32s3 target/xtensa-esp32s3-espidf/${BUILD_MODE}/ele_ds_client_rust
