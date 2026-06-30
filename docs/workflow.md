# Development Workflow

Use this as the standard loop while building the prototype.

## Quick cycle

1. Edit in `crates/game_sim` and `crates/game_app` as needed.
2. Format:
   - `cargo fmt`
3. Run tests:
   - `cargo test`
4. Run the Bevy shell:
   - `cargo run -p game_app`

Run tests first when you change simulation code, then run the app to verify render/input behavior.

## What each crate means

- `game_sim`: gameplay source of truth.
- `game_app`: Bevy shell and platform-specific glue.

## Command goals

- `cargo test`
  - Mostly validates deterministic logic and simulation behavior.
- `cargo run -p game_app`
  - Boots macOS Bevy app and checks real runtime wiring.

## Targeted platform plan

- Now: macOS-first prototype validation.
- Next: iPadOS/iOS.
- After that: Swift/Metal renderer that uses the same `game_sim` core.

