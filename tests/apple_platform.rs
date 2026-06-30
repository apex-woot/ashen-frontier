use std::path::Path;

#[test]
fn apple_packaging_builds_rust_without_the_bevy_prototype() {
    let manifest = include_str!("../Cargo.toml");
    let macos_build_script = include_str!("../apple/macos/build-rust.sh");
    let xcframework_script = include_str!("../apple/build-xcframework.sh");

    assert!(manifest.contains("bevy-prototype = [\"dep:bevy\"]"));
    assert!(manifest.contains("bevy = { version = \"0.19.0\", optional = true }"));
    assert!(macos_build_script.contains("--no-default-features"));
    assert!(xcframework_script.contains("aarch64-apple-ios-sim"));
    assert!(xcframework_script.contains("x86_64-apple-ios"));
    assert!(xcframework_script.contains("aarch64-apple-ios"));
    assert!(xcframework_script.contains("xcrun lipo"));
    assert!(xcframework_script.contains("-create-xcframework"));
    assert!(xcframework_script.contains("--no-default-features"));
}

#[test]
fn ios_simulator_project_uses_the_rust_bridge_and_metal_renderer() {
    let project = include_str!("../apple/ios/AshenFrontierIOS.xcodeproj/project.pbxproj");
    let renderer = include_str!("../apple/macos/Sources/AshenFrontierMac/Renderer.swift");
    let ios_view = include_str!("../apple/ios/AshenFrontierIOS/GameView.swift");
    let ios_controller = include_str!("../apple/ios/AshenFrontierIOS/GameViewController.swift");

    assert!(Path::new("apple/ios/AshenFrontierIOS/AppDelegate.swift").exists());
    assert!(Path::new("apple/ios/AshenFrontierIOS/GameView.swift").exists());
    assert!(Path::new("apple/ios/AshenFrontierIOS/Info.plist").exists());
    assert!(project.contains("AshenFrontierRust.xcframework"));
    assert!(project.contains("AshenFrontierBridge"));
    assert!(project.contains("AshenShaders.metal"));
    assert!(project.contains("iphoneos"));
    assert!(renderer.contains("makeDefaultLibrary"));
    assert!(renderer.contains("SWIFT_PACKAGE"));
    assert!(ios_view.contains("MTKView"));
    assert!(ios_controller.contains("RustWorld(width: 32, height: 24)"));
}

#[test]
fn ios_shell_is_portrait_first_and_maps_touch_points_into_world_space() {
    let info_plist = include_str!("../apple/ios/AshenFrontierIOS/Info.plist");
    let ios_view = include_str!("../apple/ios/AshenFrontierIOS/GameView.swift");
    let ios_controller = include_str!("../apple/ios/AshenFrontierIOS/GameViewController.swift");

    assert!(info_plist.contains("<key>UIRequiresFullScreen</key>"));
    assert!(info_plist.contains("<string>UIInterfaceOrientationPortrait</string>"));
    assert!(!info_plist.contains("UIInterfaceOrientationLandscapeLeft"));
    assert!(!info_plist.contains("UIInterfaceOrientationLandscapeRight"));
    assert!(ios_controller.contains("supportedInterfaceOrientations"));
    assert!(ios_controller.contains(".portrait"));
    assert!(ios_controller.contains("preferredInterfaceOrientationForPresentation"));

    assert!(ios_view.contains("touchPointForWorld"));
    assert!(ios_view.contains("isUserInteractionEnabled = true"));
    assert!(ios_view.contains("isMultipleTouchEnabled = true"));
    assert!(ios_view.contains("bounds.height - point.y"));
    assert!(ios_view.contains("selectUnit?(touchPointForWorld"));
    assert!(ios_view.contains("moveSelectedUnits?(touchPointForWorld"));
}

#[test]
fn ios_shell_uses_touch_specific_controls_in_the_hud() {
    let controller = include_str!("../apple/macos/Sources/AshenFrontierMac/GameController.swift");
    let ios_controller = include_str!("../apple/ios/AshenFrontierIOS/GameViewController.swift");

    assert!(controller.contains("controlHint"));
    assert!(controller.contains("Controls:"));
    assert!(ios_controller.contains("Tap=select"));
    assert!(ios_controller.contains("Long press=move"));
    assert!(ios_controller.contains("Two-finger=horde"));
}

#[test]
fn apple_shell_uses_a_shared_aspect_preserving_viewport() {
    let controller = include_str!("../apple/macos/Sources/AshenFrontierMac/GameController.swift");
    let renderer = include_str!("../apple/macos/Sources/AshenFrontierMac/Renderer.swift");

    assert!(controller.contains("struct ViewportTransform"));
    assert!(controller.contains("min(viewWidth / worldWidth"));
    assert!(controller.contains("viewPointToWorld"));
    assert!(controller.contains("worldToClip"));
    assert!(controller.contains("worldHalfSizeToClip"));
    assert!(controller.contains("panCamera"));
    assert!(controller.contains("zoomCamera"));
    assert!(controller.contains("initialCameraZoom"));
    assert!(controller.contains("clampCameraCenter"));

    assert!(renderer.contains("controller.viewport"));
    assert!(renderer.contains("viewport.worldToClip"));
    assert!(renderer.contains("viewport.worldHalfSizeToClip"));
    assert!(!renderer.contains("halfSize: SIMD2<Float>(0.018, 0.024)"));
}

#[test]
fn ios_shell_exposes_touch_camera_controls() {
    let ios_view = include_str!("../apple/ios/AshenFrontierIOS/GameView.swift");
    let ios_controller = include_str!("../apple/ios/AshenFrontierIOS/GameViewController.swift");

    assert!(ios_view.contains("UIPanGestureRecognizer"));
    assert!(ios_view.contains("UIPinchGestureRecognizer"));
    assert!(ios_view.contains("panCamera"));
    assert!(ios_view.contains("zoomCamera"));
    assert!(ios_view.contains("translationForWorld"));
    assert!(ios_view.contains("recognizer.scale = 1.0"));

    assert!(ios_controller.contains("controller.panCamera"));
    assert!(ios_controller.contains("controller.zoomCamera"));
    assert!(ios_controller.contains("initialCameraZoom: 2.0"));
    assert!(ios_controller.contains("Drag=pan"));
    assert!(ios_controller.contains("Pinch=zoom"));
}
