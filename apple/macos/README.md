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

Open `apple/macos/Package.swift` in Xcode for IDE work.

## Controls

- `H`: spawn a 64-enemy horde through the Rust C ABI.

The renderer is intentionally minimal: Swift owns the macOS window and Metal draw loop, while Rust owns the simulation state.
