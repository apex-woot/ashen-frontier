#![allow(clippy::not_unsafe_ptr_arg_deref)]

use crate::sim::{Enemy, GameWorld, GridSize, Unit};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AfEntityPosition {
    pub id: u32,
    pub x: f32,
    pub y: f32,
}

pub struct AfWorld {
    world: GameWorld,
}

#[unsafe(no_mangle)]
pub extern "C" fn af_world_create(width: u16, height: u16) -> *mut AfWorld {
    Box::into_raw(Box::new(AfWorld {
        world: GameWorld::new(GridSize::new(width, height)),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn af_world_destroy(world: *mut AfWorld) {
    if world.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(world));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn af_world_step(world: *mut AfWorld, steps: u32) {
    let Some(world) = world_mut(world) else {
        return;
    };

    world.world.step(steps);
}

#[unsafe(no_mangle)]
pub extern "C" fn af_world_spawn_horde(world: *mut AfWorld, enemy_count: u32) {
    let Some(world) = world_mut(world) else {
        return;
    };

    world
        .world
        .spawn_horde(usize::try_from(enemy_count).unwrap_or(usize::MAX));
}

#[unsafe(no_mangle)]
pub extern "C" fn af_world_tick(world: *const AfWorld) -> u64 {
    world_ref(world).map_or(0, |world| world.world.tick())
}

#[unsafe(no_mangle)]
pub extern "C" fn af_world_unit_count(world: *const AfWorld) -> usize {
    world_ref(world).map_or(0, |world| world.world.units().len())
}

#[unsafe(no_mangle)]
pub extern "C" fn af_world_enemy_count(world: *const AfWorld) -> usize {
    world_ref(world).map_or(0, |world| world.world.enemies().len())
}

#[unsafe(no_mangle)]
pub extern "C" fn af_world_read_units(
    world: *const AfWorld,
    out_positions: *mut AfEntityPosition,
    capacity: usize,
) -> usize {
    let Some(world) = world_ref(world) else {
        return 0;
    };

    write_positions(
        world.world.units().iter().map(unit_position),
        out_positions,
        capacity,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn af_world_read_enemies(
    world: *const AfWorld,
    out_positions: *mut AfEntityPosition,
    capacity: usize,
) -> usize {
    let Some(world) = world_ref(world) else {
        return 0;
    };

    write_positions(
        world.world.enemies().iter().map(enemy_position),
        out_positions,
        capacity,
    )
}

fn write_positions(
    positions: impl Iterator<Item = AfEntityPosition>,
    out_positions: *mut AfEntityPosition,
    capacity: usize,
) -> usize {
    if out_positions.is_null() || capacity == 0 {
        return 0;
    }

    let mut written = 0;
    for position in positions.take(capacity) {
        unsafe {
            out_positions.add(written).write(position);
        }
        written += 1;
    }

    written
}

fn unit_position(unit: &Unit) -> AfEntityPosition {
    AfEntityPosition {
        id: unit.id.value(),
        x: unit.position.x,
        y: unit.position.y,
    }
}

fn enemy_position(enemy: &Enemy) -> AfEntityPosition {
    AfEntityPosition {
        id: enemy.id.value(),
        x: enemy.position.x,
        y: enemy.position.y,
    }
}

fn world_ref<'world>(world: *const AfWorld) -> Option<&'world AfWorld> {
    if world.is_null() {
        return None;
    }

    unsafe { world.as_ref() }
}

fn world_mut<'world>(world: *mut AfWorld) -> Option<&'world mut AfWorld> {
    if world.is_null() {
        return None;
    }

    unsafe { world.as_mut() }
}
