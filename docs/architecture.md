# ashen-frontier Architecture (Bevy-first bootstrap)

`ashen-frontier` is a **prototype shell first, game engine second**.

Right now, the visible app is driven by `bevy` in `src/main.rs`, while the real game rules live in `src/sim.rs`.

## Two modules, two responsibilities

### `src/main.rs`
- Runs the Bevy application loop, window setup, input, rendering, and runtime startup.
- Owns all Bevy plugin wiring, startup systems, and platform entrypoints.
- Calls into `sim` for gameplay updates and world state.

### `src/sim.rs`
- Holds pure gameplay simulation logic (entities, turns, rules, combat, win conditions, state transitions).
- Should avoid direct Bevy types when possible, so it can be reused elsewhere.
- Becomes the extraction target for a later Swift/Metal runtime.

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
- Later: **iPadOS/iOS** and **Swift/Metal** renderer where the Bevy shell may be replaced, but `sim` stays and keeps gameplay behavior consistent.

## File-level expectation

- Keep this in mind when adding files:
  - `src/sim.rs` or `src/sim/...` → game data types, systems, deterministic updates.
  - `src/main.rs` or `src/app/...` → Bevy scenes, schedules, input adapters, UI/audio debug hooks.
