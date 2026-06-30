#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
HEADER_DIR="$REPO_ROOT/apple/macos/Sources/AshenFrontierBridge/include"
OUTPUT_DIR="$REPO_ROOT/target/apple"
OUTPUT_PATH="$OUTPUT_DIR/AshenFrontierRust.xcframework"
SIMULATOR_LIB_DIR="$OUTPUT_DIR/ios-simulator"
SIMULATOR_LIB="$SIMULATOR_LIB_DIR/libashen_frontier.a"

require_full_xcode() {
    if ! xcodebuild -version >/dev/null 2>&1; then
        cat >&2 <<'EOF'
Full Xcode is required to build the Apple XCFramework.

Select Xcode, then retry:
  sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
EOF
        exit 1
    fi
}

require_rust_target() {
    target="$1"
    if ! rustup target list --installed | grep -qx "$target"; then
        cat >&2 <<EOF
Missing Rust target: $target

Install it, then retry:
  rustup target add $target
EOF
        exit 1
    fi
}

build_staticlib() {
    target="$1"
    cargo build --release --lib --no-default-features --target "$target"
}

cd "$REPO_ROOT"

require_full_xcode
require_rust_target aarch64-apple-darwin
require_rust_target aarch64-apple-ios-sim
require_rust_target x86_64-apple-ios
require_rust_target aarch64-apple-ios

cargo run --quiet -p xtask -- generate-header

build_staticlib aarch64-apple-darwin
build_staticlib aarch64-apple-ios-sim
build_staticlib x86_64-apple-ios
build_staticlib aarch64-apple-ios

rm -rf "$OUTPUT_PATH"
mkdir -p "$OUTPUT_DIR"
rm -rf "$SIMULATOR_LIB_DIR"
mkdir -p "$SIMULATOR_LIB_DIR"

xcrun lipo -create \
    "$REPO_ROOT/target/aarch64-apple-ios-sim/release/libashen_frontier.a" \
    "$REPO_ROOT/target/x86_64-apple-ios/release/libashen_frontier.a" \
    -output "$SIMULATOR_LIB"

xcodebuild -create-xcframework \
    -library "$REPO_ROOT/target/aarch64-apple-darwin/release/libashen_frontier.a" \
    -headers "$HEADER_DIR" \
    -library "$SIMULATOR_LIB" \
    -headers "$HEADER_DIR" \
    -library "$REPO_ROOT/target/aarch64-apple-ios/release/libashen_frontier.a" \
    -headers "$HEADER_DIR" \
    -output "$OUTPUT_PATH"

printf 'Built %s\n' "$OUTPUT_PATH"
