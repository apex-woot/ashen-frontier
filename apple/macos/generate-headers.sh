#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)

cd "$REPO_ROOT"
cbindgen --config cbindgen.toml --crate ashen-frontier --output apple/macos/Sources/AshenFrontierBridge/include/ashen_frontier.h
