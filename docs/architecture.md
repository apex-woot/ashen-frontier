# ashen-frontier Architecture (Bevy-first bootstrap)

`ashen-frontier` is a **prototype shell first, game engine second**.

Right now, the visible app is driven by `bevy` in `crates/game_app`, while the real game rules live in `crates/game_sim`.

## Two crates, two responsibilities

### `crates/game_app`
- Runs the Bevy application loop, window setup, input, rendering, and runtime startup.
- Owns all Bevy plugin wiring, startup systems, and platform entrypoints.
- Calls into `game_sim` for gameplay updates and world state.

### `crates/game_sim`
- Holds pure gameplay simulation logic (entities, turns, rules, combat, win conditions, state transitions).
- Should avoid direct Bevy types when possible, so it can be reused elsewhere.
- Becomes the extraction target for a later Swift/Metal runtime.

## Why this split

This split gives two important properties:

1. **Fast Bevy prototyping**: artists and gameplay work can iterate in a real engine quickly.
2. **Simulation portability**: later, `game_sim` can power a non-Bevy frontend without rewriting core logic.

## Data flow (high level)

- `game_app` owns the frame loop (tick/update/render).
- On each tick, `game_app` sends player/input/system events into `game_sim`.
- `game_sim` updates authoritative state.
- `game_app` reads that state and updates Bevy entities for visual feedback.

Think of `game_sim` as the "headquarters" and `game_app` as the "front desk."

## Platform path

- First target: **macOS** (desktop prototype, single-window shell).
- Later: **iPadOS/iOS** and **Swift/Metal** renderer where `game_app` may be replaced, but `game_sim` stays and keeps gameplay behavior consistent.

## File-level expectation

- Keep this in mind when adding files:
  - `crates/game_sim/src/...` → game data types, systems, deterministic updates.
  - `crates/game_app/src/...` → Bevy scenes, schedules, input adapters, UI/audio debug hooks.
