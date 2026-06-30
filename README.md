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

The current Apple shell is a macOS target. Use Xcode's `My Mac` destination for this slice; iOS simulator support comes next.
