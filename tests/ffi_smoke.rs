use ashen_frontier::ffi::{
    AfEntityPosition, af_world_create, af_world_destroy, af_world_enemy_count,
    af_world_read_enemies, af_world_read_units, af_world_spawn_horde, af_world_step, af_world_tick,
    af_world_unit_count,
};

#[test]
fn ffi_world_steps_and_exports_positions() {
    let world = af_world_create(32, 24);
    assert!(!world.is_null());

    assert_eq!(af_world_unit_count(world), 6);
    assert_eq!(af_world_enemy_count(world), 0);

    let mut units = [AfEntityPosition::default(); 8];
    let unit_count = af_world_read_units(world, units.as_mut_ptr(), units.len());
    assert_eq!(unit_count, 6);
    assert_eq!(units[0].id, 1);

    af_world_step(world, 3);
    assert_eq!(af_world_tick(world), 3);

    af_world_spawn_horde(world, 4);
    assert_eq!(af_world_enemy_count(world), 4);

    let mut enemies = [AfEntityPosition::default(); 8];
    let enemy_count = af_world_read_enemies(world, enemies.as_mut_ptr(), enemies.len());
    assert_eq!(enemy_count, 4);
    assert_eq!(enemies[0].id, 1);

    af_world_destroy(world);
}

#[test]
fn ffi_null_world_reads_as_empty() {
    assert_eq!(af_world_unit_count(std::ptr::null()), 0);
    assert_eq!(af_world_enemy_count(std::ptr::null()), 0);
    assert_eq!(af_world_tick(std::ptr::null()), 0);
    assert_eq!(
        af_world_read_units(std::ptr::null(), std::ptr::null_mut(), 4),
        0
    );
    assert_eq!(
        af_world_read_enemies(std::ptr::null(), std::ptr::null_mut(), 4),
        0
    );
}
