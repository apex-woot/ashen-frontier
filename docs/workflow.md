# Development Workflow

Use this as the standard loop while building the prototype.

## Quick cycle

1. Edit in `src/sim.rs` and `src/main.rs` as needed.
2. Format:
   - `cargo fmt`
3. Run tests:
   - `cargo test`
4. Run the Bevy shell:
   - `cargo run`

Run tests first when you change simulation code, then run the app to verify render/input behavior.

## Apple shell cycle

1. Regenerate the Swift bridge header:
   - `./apple/macos/generate-headers.sh`
2. Build the Rust static library:
   - `./apple/macos/build-rust.sh`
3. Build the Swift/Metal shell:
   - `cd apple/macos && swift build`
4. Run the Swift/Metal shell:
   - `cd apple/macos && swift run AshenFrontierMac`

Open `apple/macos/Package.swift` in Xcode for IDE work.

## iOS simulator cycle

1. Select full Xcode if the active developer directory is still Command Line Tools:
   - `sudo xcode-select -s /Applications/Xcode.app/Contents/Developer`
2. Install Rust Apple targets once:
   - `rustup target add aarch64-apple-ios-sim x86_64-apple-ios aarch64-apple-ios`
3. Build the Rust XCFramework:
   - `./apple/build-xcframework.sh`
4. Open the iOS project:
   - `open apple/ios/AshenFrontierIOS.xcodeproj`
5. Select an iPhone or iPad simulator and run `AshenFrontierIOS`.

## What each crate means

- `src/sim.rs`: gameplay source of truth.
- `src/ffi.rs`: C ABI wrapper for Swift and later native shells.
- `cbindgen.toml`: Rust-to-C-header generation config for the Apple bridge.
- `src/main.rs`: Bevy shell and platform-specific glue.
- `apple/macos`: macOS Swift/Metal shell.
- `apple/ios`: iOS/iPadOS Swift/Metal shell.
- `apple/build-xcframework.sh`: builds `target/apple/AshenFrontierRust.xcframework` for Apple app targets.

## Command goals

- `cargo test`
  - Mostly validates deterministic logic and simulation behavior.
- `cargo run`
  - Boots macOS Bevy app and checks real runtime wiring.

## Targeted platform plan

- Now: macOS-first prototype validation.
- Current: Swift/Metal macOS and iOS shells using the Rust C ABI.
- Next: move more renderer data through bulk ABI snapshots and add platform-specific input polish.
