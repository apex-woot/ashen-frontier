#[test]
fn sim_hot_paths_avoid_libm_distance_calls() {
    let sim = include_str!("../src/sim.rs");
    let controller = include_str!("../apple/macos/Sources/AshenFrontierMac/GameController.swift");

    assert!(!sim.contains(".hypot("));
    assert!(!controller.contains("hypot("));
}

#[test]
fn sim_does_not_rebuild_unused_enemy_spatial_index() {
    let sim = include_str!("../src/sim.rs");

    assert!(!sim.contains("enemy_chunks"));
    assert!(!sim.contains("rebuild_enemy_spatial_index"));
}
