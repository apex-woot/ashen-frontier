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

1. Build the Rust static library:
   - `./apple/macos/build-rust.sh`
2. Build the Swift/Metal shell:
   - `cd apple/macos && swift build`
3. Run the Swift/Metal shell:
   - `cd apple/macos && swift run AshenFrontierMac`

Open `apple/macos/Package.swift` in Xcode for IDE work.

## What each crate means

- `src/sim.rs`: gameplay source of truth.
- `src/ffi.rs`: C ABI wrapper for Swift and later native shells.
- `src/main.rs`: Bevy shell and platform-specific glue.
- `apple/macos`: macOS Swift/Metal shell.

## Command goals

- `cargo test`
  - Mostly validates deterministic logic and simulation behavior.
- `cargo run`
  - Boots macOS Bevy app and checks real runtime wiring.

## Targeted platform plan

- Now: macOS-first prototype validation.
- Current: Swift/Metal macOS shell using the Rust C ABI.
- Next: iPadOS/iOS shell using the same `sim` core and similar platform boundary.
