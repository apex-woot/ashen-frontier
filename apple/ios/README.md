# Ashen Frontier iOS Shell

This is the first UIKit/Metal shell for the Rust simulation.

## One-time setup

Select full Xcode, install `cbindgen`, and install the Rust Apple targets:

```sh
sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
cargo install cbindgen
rustup target add aarch64-apple-ios-sim x86_64-apple-ios aarch64-apple-ios
```

## Build the Rust XCFramework

From the repo root:

```sh
./apple/build-xcframework.sh
```

The script builds `target/apple/AshenFrontierRust.xcframework` from the Rust static library for macOS, iOS simulator, and iOS device.

## Run

```sh
open apple/ios/AshenFrontierIOS.xcodeproj
```

In Xcode, choose an iPhone or iPad simulator and run the `AshenFrontierIOS` target.

The iOS shell is portrait-first. The app declares portrait as its supported orientation and requires full screen on iPad so the orientation lock is honored.

The app starts on a main menu where you can choose the map size.

## Controls

- Single tap unit: select the nearest unit.
- Single tap ground: move the selected unit through the Rust C ABI.
- Long press: area-select nearby units.
- Drag: pan the camera.
- Pinch: zoom the camera.
- Bottom buttons: select all, clear, spawn workers/rangers/soldiers, or spawn a 64-enemy horde.
