#![allow(clippy::not_unsafe_ptr_arg_deref)]

use crate::sim::{
    CommandRejection, Enemy, EnemyKind, GameWorld, GridSize, Unit, UnitId, UnitKind, WorldPoint,
};

pub const AF_COMMAND_STATUS_ACCEPTED: u32 = 0;
pub const AF_COMMAND_STATUS_EMPTY_SELECTION: u32 = 1;
pub const AF_COMMAND_STATUS_DESTINATION_OUT_OF_BOUNDS: u32 = 2;
pub const AF_COMMAND_STATUS_BLOCKED_DESTINATION: u32 = 3;
pub const AF_COMMAND_STATUS_NO_PATH: u32 = 4;
pub const AF_COMMAND_STATUS_UNKNOWN_UNIT: u32 = 5;
pub const AF_COMMAND_STATUS_INVALID_UNIT_LIST: u32 = 100;
pub const AF_UNIT_KIND_WORKER: u32 = 0;
pub const AF_UNIT_KIND_RANGER: u32 = 1;
pub const AF_UNIT_KIND_SOLDIER: u32 = 2;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AfEntityPosition {
    pub id: u32,
    pub kind: u32,
    pub health: f32,
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
pub extern "C" fn af_world_spawn_unit(world: *mut AfWorld, unit_kind: u32) -> u32 {
    let Some(world) = world_mut(world) else {
        return 0;
    };
    let Some(unit_kind) = unit_kind_from_code(unit_kind) else {
        return 0;
    };

    world
        .world
        .spawn_unit_near_command_center(unit_kind)
        .map_or(0, UnitId::value)
}

#[unsafe(no_mangle)]
pub extern "C" fn af_world_move_units(
    world: *mut AfWorld,
    unit_ids: *const u32,
    unit_count: usize,
    destination_x: f32,
    destination_y: f32,
) -> u32 {
    let Some(world) = world_mut(world) else {
        return AF_COMMAND_STATUS_INVALID_UNIT_LIST;
    };

    if unit_ids.is_null() {
        return AF_COMMAND_STATUS_INVALID_UNIT_LIST;
    }

    let unit_ids = unsafe { std::slice::from_raw_parts(unit_ids, unit_count) };
    let units = unit_ids
        .iter()
        .copied()
        .map(UnitId::new)
        .collect::<Vec<_>>();

    world
        .world
        .move_units(&units, WorldPoint::new(destination_x, destination_y))
        .map_or_else(command_rejection_code, |()| AF_COMMAND_STATUS_ACCEPTED)
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
        kind: unit_kind_code(unit.kind),
        health: unit.health,
        x: unit.position.x,
        y: unit.position.y,
    }
}

fn enemy_position(enemy: &Enemy) -> AfEntityPosition {
    AfEntityPosition {
        id: enemy.id.value(),
        kind: enemy_kind_code(enemy.kind),
        health: enemy.health,
        x: enemy.position.x,
        y: enemy.position.y,
    }
}

fn command_rejection_code(rejection: CommandRejection) -> u32 {
    match rejection {
        CommandRejection::EmptySelection => AF_COMMAND_STATUS_EMPTY_SELECTION,
        CommandRejection::DestinationOutOfBounds => AF_COMMAND_STATUS_DESTINATION_OUT_OF_BOUNDS,
        CommandRejection::BlockedDestination => AF_COMMAND_STATUS_BLOCKED_DESTINATION,
        CommandRejection::NoPath => AF_COMMAND_STATUS_NO_PATH,
        CommandRejection::UnknownUnit(_) => AF_COMMAND_STATUS_UNKNOWN_UNIT,
    }
}

fn unit_kind_from_code(unit_kind: u32) -> Option<UnitKind> {
    match unit_kind {
        AF_UNIT_KIND_WORKER => Some(UnitKind::Worker),
        AF_UNIT_KIND_RANGER => Some(UnitKind::Ranger),
        AF_UNIT_KIND_SOLDIER => Some(UnitKind::Soldier),
        _ => None,
    }
}

fn unit_kind_code(unit_kind: UnitKind) -> u32 {
    match unit_kind {
        UnitKind::Worker => AF_UNIT_KIND_WORKER,
        UnitKind::Ranger => AF_UNIT_KIND_RANGER,
        UnitKind::Soldier => AF_UNIT_KIND_SOLDIER,
    }
}

fn enemy_kind_code(enemy_kind: EnemyKind) -> u32 {
    match enemy_kind {
        EnemyKind::InfectedDecrepit => 0,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffi_spawns_player_unit_and_reports_new_count() {
        let world = af_world_create(32, 24);

        let unit_id = af_world_spawn_unit(world, AF_UNIT_KIND_RANGER);
        let mut positions = [AfEntityPosition::default(); 8];
        let written = af_world_read_units(world, positions.as_mut_ptr(), positions.len());
        let spawned = positions
            .iter()
            .take(written)
            .find(|position| position.id == unit_id)
            .expect("spawned unit should be readable");

        assert_eq!(unit_id, 7);
        assert_eq!(af_world_unit_count(world), 7);
        assert_eq!(spawned.kind, AF_UNIT_KIND_RANGER);
        assert!((spawned.health - UnitKind::Ranger.stats().max_health).abs() < f32::EPSILON);

        af_world_destroy(world);
    }

    #[test]
    fn ffi_rejects_unknown_unit_kind() {
        let world = af_world_create(32, 24);

        let unit_id = af_world_spawn_unit(world, u32::MAX);

        assert_eq!(unit_id, 0);
        assert_eq!(af_world_unit_count(world), 6);

        af_world_destroy(world);
    }
}
