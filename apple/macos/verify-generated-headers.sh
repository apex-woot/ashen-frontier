#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)

cd "$REPO_ROOT"
tmp_header=$(mktemp)
trap 'rm -f "$tmp_header"' EXIT

cbindgen --config cbindgen.toml --crate ashen-frontier --output "$tmp_header"
cmp -s apple/macos/Sources/AshenFrontierBridge/include/ashen_frontier.h "$tmp_header"
