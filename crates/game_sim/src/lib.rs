const STARTING_WORKERS: usize = 6;
const WORKER_SPEED_PER_TICK: f32 = 1.5;
const GROUP_MOVE_SPACING: f32 = 0.8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimConfig {
    pub grid_size: GridSize,
}

impl SimConfig {
    #[must_use]
    pub const fn new(width: u16, height: u16) -> Self {
        Self {
            grid_size: GridSize::new(width, height),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridSize {
    pub width: u16,
    pub height: u16,
}

impl GridSize {
    #[must_use]
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    #[must_use]
    pub fn cell_count(self) -> usize {
        usize::from(self.width) * usize::from(self.height)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridCoord {
    pub x: u16,
    pub y: u16,
}

impl GridCoord {
    #[must_use]
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainCell {
    Clear,
    Blocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainError {
    OutOfBounds(GridCoord),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldPoint {
    pub x: f32,
    pub y: f32,
}

impl WorldPoint {
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldRect {
    min: WorldPoint,
    max: WorldPoint,
}

impl WorldRect {
    #[must_use]
    pub fn from_corners(first: WorldPoint, second: WorldPoint) -> Self {
        Self {
            min: WorldPoint::new(first.x.min(second.x), first.y.min(second.y)),
            max: WorldPoint::new(first.x.max(second.x), first.y.max(second.y)),
        }
    }

    #[must_use]
    pub fn contains(self, point: WorldPoint) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UnitId(u32);

impl UnitId {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BuildingId(u32);

impl BuildingId {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitKind {
    Worker,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Unit {
    pub id: UnitId,
    pub kind: UnitKind,
    pub position: WorldPoint,
    pub target: Option<WorldPoint>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildingKind {
    CommandCenter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Building {
    pub id: BuildingId,
    pub kind: BuildingKind,
    pub position: GridCoord,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    MoveUnits {
        units: Vec<UnitId>,
        destination: WorldPoint,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandResult {
    Accepted,
    Rejected(CommandRejection),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandRejection {
    EmptySelection,
    DestinationOutOfBounds,
    BlockedDestination,
    UnknownUnit(UnitId),
}

#[derive(Clone, Debug)]
pub struct GameWorld {
    config: SimConfig,
    seed: u64,
    tick: u64,
    units: Vec<Unit>,
    buildings: Vec<Building>,
    terrain: Vec<TerrainCell>,
}

impl GameWorld {
    #[must_use]
    pub fn new(config: SimConfig, seed: u64) -> Self {
        let center = GridCoord::new(config.grid_size.width / 2, config.grid_size.height / 2);
        let mut world = Self {
            config,
            seed,
            tick: 0,
            units: Vec::with_capacity(STARTING_WORKERS),
            buildings: vec![Building {
                id: BuildingId::new(1),
                kind: BuildingKind::CommandCenter,
                position: center,
            }],
            terrain: vec![TerrainCell::Clear; config.grid_size.cell_count()],
        };

        world.spawn_starting_workers(center);
        world
    }

    #[must_use]
    pub const fn grid_size(&self) -> GridSize {
        self.config.grid_size
    }

    #[must_use]
    pub const fn tick(&self) -> u64 {
        self.tick
    }

    #[must_use]
    pub fn units(&self) -> &[Unit] {
        &self.units
    }

    #[must_use]
    pub fn buildings(&self) -> &[Building] {
        &self.buildings
    }

    #[must_use]
    pub fn terrain_at(&self, coord: GridCoord) -> Option<TerrainCell> {
        self.terrain_index(coord)
            .and_then(|index| self.terrain.get(index).copied())
    }

    /// Sets one terrain cell.
    ///
    /// # Errors
    ///
    /// Returns [`TerrainError::OutOfBounds`] when `coord` is outside the map.
    pub fn set_terrain(
        &mut self,
        coord: GridCoord,
        terrain: TerrainCell,
    ) -> Result<(), TerrainError> {
        let Some(index) = self.terrain_index(coord) else {
            return Err(TerrainError::OutOfBounds(coord));
        };

        if let Some(cell) = self.terrain.get_mut(index) {
            *cell = terrain;
        }

        Ok(())
    }

    /// Sets terrain at a world-space point.
    ///
    /// # Errors
    ///
    /// Returns [`TerrainError::OutOfBounds`] when `point` is outside the map.
    pub fn set_terrain_at_point(
        &mut self,
        point: WorldPoint,
        terrain: TerrainCell,
    ) -> Result<(), TerrainError> {
        let coord = self.world_point_to_grid(point).unwrap_or(GridCoord::new(
            self.config.grid_size.width,
            self.config.grid_size.height,
        ));
        self.set_terrain(coord, terrain)
    }

    #[must_use]
    pub fn blocked_cell_count(&self) -> usize {
        self.terrain
            .iter()
            .filter(|cell| **cell == TerrainCell::Blocked)
            .count()
    }

    #[must_use]
    pub fn blocked_cells(&self) -> Vec<GridCoord> {
        self.terrain
            .iter()
            .enumerate()
            .filter_map(|(index, cell)| {
                if *cell == TerrainCell::Blocked {
                    self.grid_coord_from_index(index)
                } else {
                    None
                }
            })
            .collect()
    }

    #[must_use]
    pub fn unit(&self, id: UnitId) -> Option<&Unit> {
        self.units.iter().find(|unit| unit.id == id)
    }

    #[must_use]
    pub fn select_units(&self, rect: WorldRect) -> Vec<UnitId> {
        self.units
            .iter()
            .filter(|unit| rect.contains(unit.position))
            .map(|unit| unit.id)
            .collect()
    }

    pub fn submit_command(&mut self, command: Command) -> CommandResult {
        match command {
            Command::MoveUnits { units, destination } => {
                if units.is_empty() {
                    return CommandResult::Rejected(CommandRejection::EmptySelection);
                }

                if !self.contains_world_point(destination) {
                    return CommandResult::Rejected(CommandRejection::DestinationOutOfBounds);
                }

                if self.is_world_point_blocked(destination) {
                    return CommandResult::Rejected(CommandRejection::BlockedDestination);
                }

                if let Some(unknown_id) = units.iter().copied().find(|id| self.unit(*id).is_none())
                {
                    return CommandResult::Rejected(CommandRejection::UnknownUnit(unknown_id));
                }

                let unit_count = units.len();
                for (index, unit_id) in units.into_iter().enumerate() {
                    let target = group_move_target(
                        index,
                        unit_count,
                        destination,
                        self.config.grid_size,
                        GROUP_MOVE_SPACING,
                    );
                    let Some(unit) = self.units.iter_mut().find(|unit| unit.id == unit_id) else {
                        return CommandResult::Rejected(CommandRejection::UnknownUnit(unit_id));
                    };
                    unit.target = Some(target);
                }

                CommandResult::Accepted
            }
        }
    }

    pub fn step(&mut self, steps: u32) {
        for _ in 0..steps {
            self.tick = self.tick.saturating_add(1);
            for unit in &mut self.units {
                move_unit_toward_target(unit, &self.terrain, self.config.grid_size);
            }
        }
    }

    pub fn set_worker_count(&mut self, worker_count: usize) {
        self.units.clear();

        let width = usize::from(self.config.grid_size.width);
        let height = usize::from(self.config.grid_size.height);
        if width == 0 || height == 0 {
            return;
        }

        let cell_count = width * height;
        self.units.reserve(worker_count);

        for index in 0..worker_count {
            let cell_index = index % cell_count;
            let lap = index / cell_count;
            let x = u16::try_from(cell_index % width).unwrap_or(0);
            let y = u16::try_from(cell_index / width).unwrap_or(0);
            let jitter_x = stress_jitter(lap % 4);
            let jitter_y = stress_jitter((lap / 4) % 4);

            self.units.push(Unit {
                id: UnitId::new(u32::try_from(index + 1).unwrap_or(u32::MAX)),
                kind: UnitKind::Worker,
                position: WorldPoint::new(f32::from(x) + jitter_x, f32::from(y) + jitter_y),
                target: None,
            });
        }
    }

    #[must_use]
    pub fn history_hash(&self) -> u64 {
        let mut hash = StableHasher::default();
        hash.write_u64(self.seed);
        hash.write_u64(self.tick);
        hash.write_u32(u32::from(self.config.grid_size.width));
        hash.write_u32(u32::from(self.config.grid_size.height));

        for building in &self.buildings {
            hash.write_u32(building.id.value());
            hash.write_u32(match building.kind {
                BuildingKind::CommandCenter => 1,
            });
            hash.write_u32(u32::from(building.position.x));
            hash.write_u32(u32::from(building.position.y));
        }

        for cell in &self.terrain {
            hash.write_u32(match cell {
                TerrainCell::Clear => 0,
                TerrainCell::Blocked => 1,
            });
        }

        for unit in &self.units {
            hash.write_u32(unit.id.value());
            hash.write_u32(match unit.kind {
                UnitKind::Worker => 1,
            });
            hash.write_f32(unit.position.x);
            hash.write_f32(unit.position.y);
            if let Some(target) = unit.target {
                hash.write_u32(1);
                hash.write_f32(target.x);
                hash.write_f32(target.y);
            } else {
                hash.write_u32(0);
            }
        }

        hash.finish()
    }

    fn spawn_starting_workers(&mut self, center: GridCoord) {
        const OFFSETS: [(f32, f32); STARTING_WORKERS] = [
            (-2.0, -1.0),
            (-1.0, -2.0),
            (1.0, -2.0),
            (2.0, -1.0),
            (-1.0, 1.0),
            (1.0, 1.0),
        ];

        for (index, (x_offset, y_offset)) in OFFSETS.into_iter().enumerate() {
            self.units.push(Unit {
                id: UnitId::new(u32::try_from(index + 1).expect("starting worker id fits in u32")),
                kind: UnitKind::Worker,
                position: WorldPoint::new(
                    f32::from(center.x) + x_offset,
                    f32::from(center.y) + y_offset,
                ),
                target: None,
            });
        }
    }

    fn contains_world_point(&self, point: WorldPoint) -> bool {
        point.x >= 0.0
            && point.y >= 0.0
            && point.x < f32::from(self.config.grid_size.width)
            && point.y < f32::from(self.config.grid_size.height)
    }

    fn is_world_point_blocked(&self, point: WorldPoint) -> bool {
        self.world_point_to_grid(point)
            .and_then(|coord| self.terrain_at(coord))
            == Some(TerrainCell::Blocked)
    }

    fn world_point_to_grid(&self, point: WorldPoint) -> Option<GridCoord> {
        world_point_to_grid(point, self.config.grid_size)
    }

    fn terrain_index(&self, coord: GridCoord) -> Option<usize> {
        terrain_index(coord, self.config.grid_size)
    }

    fn grid_coord_from_index(&self, index: usize) -> Option<GridCoord> {
        let width = usize::from(self.config.grid_size.width);
        if width == 0 || index >= self.terrain.len() {
            return None;
        }

        Some(GridCoord::new(
            u16::try_from(index % width).ok()?,
            u16::try_from(index / width).ok()?,
        ))
    }
}

fn stress_jitter(slot: usize) -> f32 {
    0.23 + f32::from(u8::try_from(slot).unwrap_or(0)) * 0.18
}

fn group_move_target(
    index: usize,
    unit_count: usize,
    destination: WorldPoint,
    grid_size: GridSize,
    spacing: f32,
) -> WorldPoint {
    if unit_count <= 1 {
        return destination;
    }

    let columns = square_root_ceil(unit_count);
    let rows = unit_count.div_ceil(columns);
    let column = index % columns;
    let row = index / columns;
    let x_offset = (small_usize_to_f32(column)
        - small_usize_to_f32(columns.saturating_sub(1)) / 2.0)
        * spacing;
    let y_offset =
        (small_usize_to_f32(row) - small_usize_to_f32(rows.saturating_sub(1)) / 2.0) * spacing;

    clamp_world_point(
        WorldPoint::new(destination.x + x_offset, destination.y + y_offset),
        grid_size,
    )
}

fn square_root_ceil(value: usize) -> usize {
    let mut root = 1;
    while root * root < value {
        root += 1;
    }
    root
}

fn small_usize_to_f32(value: usize) -> f32 {
    f32::from(u16::try_from(value).unwrap_or(u16::MAX))
}

fn clamp_world_point(point: WorldPoint, grid_size: GridSize) -> WorldPoint {
    let max_x = (f32::from(grid_size.width) - f32::EPSILON).max(0.0);
    let max_y = (f32::from(grid_size.height) - f32::EPSILON).max(0.0);

    WorldPoint::new(point.x.clamp(0.0, max_x), point.y.clamp(0.0, max_y))
}

fn move_unit_toward_target(unit: &mut Unit, terrain: &[TerrainCell], grid_size: GridSize) {
    let Some(target) = unit.target else {
        return;
    };

    let dx = target.x - unit.position.x;
    let dy = target.y - unit.position.y;
    let distance = dx.hypot(dy);

    if distance <= WORKER_SPEED_PER_TICK {
        if is_point_blocked(target, terrain, grid_size) {
            unit.target = None;
            return;
        }

        unit.position = target;
        unit.target = None;
        return;
    }

    let scale = WORKER_SPEED_PER_TICK / distance;
    let next_position = WorldPoint::new(unit.position.x + dx * scale, unit.position.y + dy * scale);
    if is_point_blocked(next_position, terrain, grid_size) {
        unit.target = None;
        return;
    }

    unit.position = next_position;
}

fn is_point_blocked(point: WorldPoint, terrain: &[TerrainCell], grid_size: GridSize) -> bool {
    let Some(coord) = world_point_to_grid(point, grid_size) else {
        return false;
    };
    let Some(index) = terrain_index(coord, grid_size) else {
        return false;
    };

    terrain.get(index).copied() == Some(TerrainCell::Blocked)
}

fn world_point_to_grid(point: WorldPoint, grid_size: GridSize) -> Option<GridCoord> {
    if point.x < 0.0
        || point.y < 0.0
        || point.x >= f32::from(grid_size.width)
        || point.y >= f32::from(grid_size.height)
    {
        return None;
    }

    Some(GridCoord::new(
        bounded_floor_to_u16(point.x),
        bounded_floor_to_u16(point.y),
    ))
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn bounded_floor_to_u16(value: f32) -> u16 {
    let floored = value.floor();
    if floored <= 0.0 {
        return 0;
    }

    let capped = floored.min(f32::from(u16::MAX));
    capped as u16
}

fn terrain_index(coord: GridCoord, grid_size: GridSize) -> Option<usize> {
    if coord.x >= grid_size.width || coord.y >= grid_size.height {
        return None;
    }

    Some(usize::from(coord.y) * usize::from(grid_size.width) + usize::from(coord.x))
}

#[derive(Default)]
struct StableHasher {
    state: u64,
}

impl StableHasher {
    fn write_u32(&mut self, value: u32) {
        self.write_u64(u64::from(value));
    }

    fn write_u64(&mut self, value: u64) {
        let mixed = value
            .wrapping_add(0x9e37_79b9_7f4a_7c15)
            .wrapping_add(self.state << 6)
            .wrapping_add(self.state >> 2);
        self.state ^= mixed;
    }

    fn write_f32(&mut self, value: f32) {
        self.write_u32(value.to_bits());
    }

    const fn finish(self) -> u64 {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_world_contains_command_center_and_workers() {
        let world = GameWorld::new(SimConfig::new(32, 24), 7);

        assert_eq!(world.grid_size(), GridSize::new(32, 24));
        assert_eq!(world.tick(), 0);
        assert_eq!(world.units().len(), 6);
        assert_eq!(world.buildings().len(), 1);
        assert!(world.buildings().iter().any(|building| {
            building.kind == BuildingKind::CommandCenter
                && building.position == GridCoord::new(16, 12)
        }));
    }

    #[test]
    fn accepted_move_command_advances_units_on_fixed_tick() {
        let mut world = GameWorld::new(SimConfig::new(32, 24), 7);
        let unit_id = world.units()[0].id;
        let start = world.units()[0].position;

        let result = world.submit_command(Command::MoveUnits {
            units: vec![unit_id],
            destination: WorldPoint::new(start.x + 3.0, start.y),
        });
        world.step(1);

        let unit = world.unit(unit_id).expect("unit should still exist");
        assert_eq!(result, CommandResult::Accepted);
        assert_eq!(world.tick(), 1);
        assert!(unit.position.x > start.x);
        assert!((unit.position.y - start.y).abs() < f32::EPSILON);
    }

    #[test]
    fn multi_unit_move_assigns_spaced_formation_targets() {
        let mut world = GameWorld::new(SimConfig::new(32, 24), 7);
        let units = world
            .units()
            .iter()
            .take(4)
            .map(|unit| unit.id)
            .collect::<Vec<_>>();

        let result = world.submit_command(Command::MoveUnits {
            units: units.clone(),
            destination: WorldPoint::new(16.0, 12.0),
        });

        assert_eq!(result, CommandResult::Accepted);
        let targets = units
            .iter()
            .map(|unit_id| world.unit(*unit_id).and_then(|unit| unit.target).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(targets[0], WorldPoint::new(15.6, 11.6));
        assert_eq!(targets[1], WorldPoint::new(16.4, 11.6));
        assert_eq!(targets[2], WorldPoint::new(15.6, 12.4));
        assert_eq!(targets[3], WorldPoint::new(16.4, 12.4));
    }

    #[test]
    fn selection_uses_world_space_rectangle_without_bevy_types() {
        let world = GameWorld::new(SimConfig::new(32, 24), 7);

        let selected = world.select_units(WorldRect::from_corners(
            WorldPoint::new(13.0, 9.0),
            WorldPoint::new(19.0, 15.0),
        ));

        assert_eq!(selected.len(), 6);
    }

    #[test]
    fn terrain_cells_can_be_blocked_and_counted() {
        let mut world = GameWorld::new(SimConfig::new(32, 24), 7);
        let coord = GridCoord::new(4, 5);

        assert_eq!(world.terrain_at(coord), Some(TerrainCell::Clear));
        assert_eq!(world.blocked_cell_count(), 0);

        assert_eq!(world.set_terrain(coord, TerrainCell::Blocked), Ok(()));

        assert_eq!(world.terrain_at(coord), Some(TerrainCell::Blocked));
        assert_eq!(world.blocked_cell_count(), 1);
    }

    #[test]
    fn move_command_rejects_blocked_destination_without_mutating_units() {
        let mut world = GameWorld::new(SimConfig::new(32, 24), 7);
        let unit_id = world.units()[0].id;
        let before = world.history_hash();

        assert_eq!(
            world.set_terrain(GridCoord::new(18, 12), TerrainCell::Blocked),
            Ok(())
        );
        let result = world.submit_command(Command::MoveUnits {
            units: vec![unit_id],
            destination: WorldPoint::new(18.2, 12.2),
        });

        assert_eq!(
            result,
            CommandResult::Rejected(CommandRejection::BlockedDestination)
        );
        assert_eq!(world.unit(unit_id).and_then(|unit| unit.target), None);
        assert_ne!(world.history_hash(), before);
    }

    #[test]
    fn unit_stops_before_entering_blocked_cell() {
        let mut world = GameWorld::new(SimConfig::new(32, 24), 7);
        let unit_id = world.units()[0].id;
        let start = world.units()[0].position;
        let blocked = GridCoord::new(
            bounded_floor_to_u16(start.x + 1.0),
            bounded_floor_to_u16(start.y),
        );

        assert_eq!(world.set_terrain(blocked, TerrainCell::Blocked), Ok(()));
        assert_eq!(
            world.submit_command(Command::MoveUnits {
                units: vec![unit_id],
                destination: WorldPoint::new(start.x + 2.0, start.y),
            }),
            CommandResult::Accepted
        );
        world.step(1);

        let unit = world.unit(unit_id).expect("unit should still exist");
        assert_eq!(unit.position, start);
        assert_eq!(unit.target, None);
    }

    #[test]
    fn rejected_command_does_not_mutate_state() {
        let mut world = GameWorld::new(SimConfig::new(32, 24), 7);
        let before = world.history_hash();

        let result = world.submit_command(Command::MoveUnits {
            units: vec![UnitId::new(404)],
            destination: WorldPoint::new(4.0, 4.0),
        });

        assert_eq!(
            result,
            CommandResult::Rejected(CommandRejection::UnknownUnit(UnitId::new(404)))
        );
        assert_eq!(world.history_hash(), before);
    }

    #[test]
    fn same_seed_and_commands_produce_same_hash() {
        let mut first = GameWorld::new(SimConfig::new(32, 24), 7);
        let mut second = GameWorld::new(SimConfig::new(32, 24), 7);
        let unit_id = first.units()[0].id;
        let destination = WorldPoint::new(18.0, 12.0);

        assert_eq!(unit_id, second.units()[0].id);
        assert_eq!(
            first.submit_command(Command::MoveUnits {
                units: vec![unit_id],
                destination,
            }),
            CommandResult::Accepted
        );
        assert_eq!(
            second.submit_command(Command::MoveUnits {
                units: vec![unit_id],
                destination,
            }),
            CommandResult::Accepted
        );

        first.step(4);
        second.step(4);

        assert_eq!(first.history_hash(), second.history_hash());
    }

    #[test]
    fn stress_population_sets_exact_worker_count() {
        let mut world = GameWorld::new(SimConfig::new(32, 24), 7);

        world.set_worker_count(1_000);

        assert_eq!(world.units().len(), 1_000);
        assert!(
            world
                .units()
                .iter()
                .all(|unit| unit.kind == UnitKind::Worker)
        );
    }

    #[test]
    fn stress_population_is_deterministic_and_in_bounds() {
        let mut first = GameWorld::new(SimConfig::new(32, 24), 7);
        let mut second = GameWorld::new(SimConfig::new(32, 24), 7);

        first.set_worker_count(5_000);
        second.set_worker_count(5_000);

        assert_eq!(first.history_hash(), second.history_hash());
        for (index, unit) in first.units().iter().enumerate() {
            assert_eq!(unit.id, UnitId::new(u32::try_from(index + 1).unwrap()));
            assert!(unit.position.x >= 0.0);
            assert!(unit.position.y >= 0.0);
            assert!(unit.position.x < f32::from(first.grid_size().width));
            assert!(unit.position.y < f32::from(first.grid_size().height));
        }
    }
}
