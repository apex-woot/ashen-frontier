# ashen-frontier Architecture (Bevy-first bootstrap)

`ashen-frontier` is a **prototype shell first, game engine second**.

Right now, the visible prototype app is driven by `bevy` in `src/main.rs`, while the real game rules live in `src/sim.rs`. A first macOS Swift/Metal shell lives under `apple/macos` and talks to the same Rust simulation through `src/ffi.rs`.

## Two modules, two responsibilities

### `src/main.rs`
- Runs the Bevy application loop, window setup, input, rendering, and runtime startup.
- Owns all Bevy plugin wiring, startup systems, and platform entrypoints.
- Calls into `sim` for gameplay updates and world state.

### `src/sim.rs`
- Holds pure gameplay simulation logic (entities, turns, rules, combat, win conditions, state transitions).
- Should avoid direct Bevy types when possible, so it can be reused elsewhere.
- Is consumed by both the Bevy prototype and the Swift/Metal shell.

### `src/ffi.rs`
- Exposes a small C ABI for Apple callers.
- Owns the unsafe raw-pointer boundary.
- Keeps Swift away from Rust internals by exporting flat position snapshots.
- Feeds `cbindgen`, which generates the Swift bridge header.

### `apple/macos`
- Swift Package that can be opened in Xcode.
- Owns the macOS window, AppKit lifecycle, MetalKit view, and Metal renderer.
- Links against the Rust static library built without the Bevy prototype feature.
- Imports the Rust ABI through a generated C header and SwiftPM module map.

### `apple/ios`
- Xcode project for iPhone and iPad simulator/device development.
- Owns UIKit lifecycle, touch input, and the iOS MetalKit view.
- Reuses the current Swift Rust wrapper, game controller, renderer, and Metal shader source from the macOS shell.
- Links against `target/apple/AshenFrontierRust.xcframework`, built from the Rust static library for macOS, iOS simulator, and iOS device.

## Why this split

This split gives two important properties without paying for a second package yet:

1. **Fast Bevy prototyping**: artists and gameplay work can iterate in a real engine quickly.
2. **Simulation portability**: later, `sim` can move into its own crate when a non-Bevy frontend exists.

## Data flow (high level)

- `src/main.rs` owns the frame loop (tick/update/render).
- On each tick, it sends player/input/system events into `sim`.
- `sim` updates authoritative state.
- `src/main.rs` reads that state and updates Bevy entities for visual feedback.

Think of `sim` as the "headquarters" and `src/main.rs` as the "front desk."

## Platform path

- First target: **macOS** (desktop prototype, single-window shell).
- Current Apple spike: **macOS and iOS Swift/Metal** shells through the Rust C ABI.
- Later: dedicated iPadOS/iOS input, app lifecycle, and platform services layered on the same ABI shape.

## File-level expectation

- Keep this in mind when adding files:
  - `src/sim.rs` or `src/sim/...` → game data types, systems, deterministic updates.
  - `src/ffi.rs` → C ABI boundary for Apple/native shells.
  - `src/main.rs` or `src/app/...` → Bevy scenes, schedules, input adapters, UI/audio debug hooks.
  - `apple/macos/...` → Swift, AppKit, MetalKit, shared Swift renderer/controller code, and `.metal` shader code.
  - `apple/ios/...` → Swift, UIKit, iOS Xcode project, and touch input code.
