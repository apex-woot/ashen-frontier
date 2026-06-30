use ashen_frontier::ffi::{
    AF_COMMAND_STATUS_ACCEPTED, AF_COMMAND_STATUS_INVALID_UNIT_LIST, AfEntityPosition,
    af_world_create, af_world_destroy, af_world_enemy_count, af_world_move_units,
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
fn ffi_move_units_accepts_selected_unit_ids() {
    let world = af_world_create(32, 24);
    assert!(!world.is_null());

    let mut units = [AfEntityPosition::default(); 8];
    let unit_count = af_world_read_units(world, units.as_mut_ptr(), units.len());
    assert_eq!(unit_count, 6);
    let start = units[0];
    let selected_units = [start.id];

    let status = af_world_move_units(
        world,
        selected_units.as_ptr(),
        selected_units.len(),
        start.x + 3.0,
        start.y,
    );
    assert_eq!(status, AF_COMMAND_STATUS_ACCEPTED);

    af_world_step(world, 2);

    let unit_count = af_world_read_units(world, units.as_mut_ptr(), units.len());
    assert_eq!(unit_count, 6);
    assert!(units[0].x > start.x);

    af_world_destroy(world);
}

#[test]
fn ffi_null_world_reads_as_empty() {
    assert_eq!(af_world_unit_count(std::ptr::null()), 0);
    assert_eq!(af_world_enemy_count(std::ptr::null()), 0);
    assert_eq!(af_world_tick(std::ptr::null()), 0);
    assert_eq!(
        af_world_move_units(std::ptr::null_mut(), std::ptr::null(), 0, 1.0, 1.0),
        AF_COMMAND_STATUS_INVALID_UNIT_LIST
    );
    assert_eq!(
        af_world_read_units(std::ptr::null(), std::ptr::null_mut(), 4),
        0
    );
    assert_eq!(
        af_world_read_enemies(std::ptr::null(), std::ptr::null_mut(), 4),
        0
    );
}
