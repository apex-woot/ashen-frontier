# ashen-frontier

Native-first RTS prototype.

## Run the Bevy prototype

```sh
cargo run
```

## Run the macOS Swift/Metal shell

```sh
./apple/macos/generate-headers.sh
./apple/macos/build-rust.sh
cd apple/macos
swift run AshenFrontierMac
```

Open `apple/macos/Package.swift` in Xcode for Apple-side development.

Use Xcode's `My Mac` destination for this slice.

## Run the iOS simulator shell

```sh
sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
rustup target add aarch64-apple-ios-sim x86_64-apple-ios aarch64-apple-ios
./apple/build-xcframework.sh
open apple/ios/AshenFrontierIOS.xcodeproj
```

In Xcode, select an iPhone or iPad simulator and run the `AshenFrontierIOS` target.
