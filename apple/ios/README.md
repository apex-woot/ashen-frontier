# Ashen Frontier iOS Shell

This is the first UIKit/Metal shell for the Rust simulation.

## One-time setup

Select full Xcode and install the Rust Apple targets:

```sh
sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
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

The app starts on a main menu. Use Settings to switch the target frame rate between 60 FPS and 120 FPS; the game applies the setting to the Metal view and HUD refresh loop.

## Controls

- Single tap: select the nearest unit.
- Long press: move the selected unit through the Rust C ABI.
- Drag: pan the camera.
- Pinch: zoom the camera.
- Two-finger tap: spawn a 64-enemy horde through the Rust C ABI.
