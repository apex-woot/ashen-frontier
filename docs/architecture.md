# ashen-frontier Architecture

`ashen-frontier` is a **Rust simulation with native Apple shells**.

The game rules live in `src/sim.rs`. macOS and iOS Swift/Metal shells talk to that same Rust simulation through `src/ffi.rs`.

## Modules

### `src/sim.rs`
- Holds pure gameplay simulation logic (entities, turns, rules, combat, win conditions, state transitions).
- Avoids platform UI types so it can be reused by each shell.

### `src/ffi.rs`
- Exposes a small C ABI for Apple callers.
- Owns the unsafe raw-pointer boundary.
- Keeps Swift away from Rust internals by exporting flat position snapshots.
- Feeds `cbindgen`, which generates the Swift bridge header.

### `apple/macos`
- Swift Package that can be opened in Xcode.
- Owns the macOS window, AppKit lifecycle, MetalKit view, and Metal renderer.
- Links against the Rust static library.
- Imports the Rust ABI through a generated C header and SwiftPM module map.

### `apple/ios`
- Xcode project for iPhone and iPad simulator/device development.
- Owns UIKit lifecycle, touch input, and the iOS MetalKit view.
- Reuses the current Swift Rust wrapper, game controller, renderer, and Metal shader source from the macOS shell.
- Links against `target/apple/AshenFrontierRust.xcframework`, built from the Rust static library for macOS, iOS simulator, and iOS device.

## Why this split

This keeps the prototype small: Rust owns the authoritative simulation, and Apple code owns native windows, input, and Metal rendering.

## Data flow (high level)

- Apple shells own the frame loop (tick/update/render).
- On each fixed tick, they send player/input/system events into `sim` through `src/ffi.rs`.
- `sim` updates authoritative state.
- The shells read flat snapshots from Rust and update native render buffers.

## Platform path

- First target: **macOS** (desktop prototype, single-window shell).
- Current Apple spike: **macOS and iOS Swift/Metal** shells through the Rust C ABI.
- Later: dedicated iPadOS/iOS input, app lifecycle, and platform services layered on the same ABI shape.

## File-level expectation

- Keep this in mind when adding files:
  - `src/sim.rs` or `src/sim/...` → game data types, systems, deterministic updates.
  - `src/ffi.rs` → C ABI boundary for Apple/native shells.
  - `apple/macos/...` → Swift, AppKit, MetalKit, shared Swift renderer/controller code, and `.metal` shader code.
  - `apple/ios/...` → Swift, UIKit, iOS Xcode project, and touch input code.
