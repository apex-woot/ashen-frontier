# Ashen Frontier macOS Shell

This is the first Swift/Metal shell for the Rust simulation.

## Build

From the repo root:

```sh
./apple/macos/build-rust.sh
cd apple/macos
swift build
swift run AshenFrontierMac
```

Open `apple/macos/Package.swift` in Xcode for IDE work. Select the macOS destination (`My Mac`) for this shell; an iOS simulator app target is the next platform slice.

## Controls

- `H`: spawn a 64-enemy horde through the Rust C ABI.
- Left click: select the nearest unit.
- Right click: move the selected unit through the Rust C ABI.

The renderer is intentionally minimal: Swift owns the macOS window and Metal draw loop, while Rust owns the simulation state.
