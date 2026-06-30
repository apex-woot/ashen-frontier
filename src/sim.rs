use std::collections::VecDeque;

const STARTING_WORKERS: usize = 6;
const WORKER_SPEED_PER_TICK: f32 = 1.5;
const ENEMY_SPEED_PER_TICK: f32 = 0.9;
const GROUP_MOVE_SPACING: f32 = 0.8;
const PATH_NEIGHBORS: [(i16, i16); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

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

#[derive(Clone, Debug, PartialEq)]
pub struct Unit {
    pub id: UnitId,
    pub position: WorldPoint,
    path: Vec<WorldPoint>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EnemyId(u32);

impl EnemyId {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Enemy {
    pub id: EnemyId,
    pub position: WorldPoint,
    path: Vec<WorldPoint>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Building {
    pub position: GridCoord,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandRejection {
    EmptySelection,
    DestinationOutOfBounds,
    BlockedDestination,
    NoPath,
    UnknownUnit(UnitId),
}

#[derive(Clone, Debug, PartialEq)]
pub struct GameWorld {
    grid_size: GridSize,
    tick: u64,
    units: Vec<Unit>,
    enemies: Vec<Enemy>,
    next_enemy_id: u32,
    buildings: Vec<Building>,
    terrain: Vec<TerrainCell>,
}

impl GameWorld {
    #[must_use]
    pub fn new(grid_size: GridSize) -> Self {
        let center = GridCoord::new(grid_size.width / 2, grid_size.height / 2);
        let mut world = Self {
            grid_size,
            tick: 0,
            units: Vec::with_capacity(STARTING_WORKERS),
            enemies: Vec::new(),
            next_enemy_id: 1,
            buildings: vec![Building { position: center }],
            terrain: vec![TerrainCell::Clear; grid_size.cell_count()],
        };

        world.spawn_starting_workers(center);
        world
    }

    #[must_use]
    pub const fn grid_size(&self) -> GridSize {
        self.grid_size
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
    pub fn enemies(&self) -> &[Enemy] {
        &self.enemies
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
        let coord = self
            .world_point_to_grid(point)
            .unwrap_or(GridCoord::new(self.grid_size.width, self.grid_size.height));
        self.set_terrain(coord, terrain)
    }

    #[must_use]
    pub fn blocked_cell_count(&self) -> usize {
        self.terrain
            .iter()
            .filter(|cell| **cell == TerrainCell::Blocked)
            .count()
    }

    pub fn blocked_cells(&self) -> impl Iterator<Item = GridCoord> + '_ {
        self.terrain.iter().enumerate().filter_map(|(index, cell)| {
            if *cell == TerrainCell::Blocked {
                self.grid_coord_from_index(index)
            } else {
                None
            }
        })
    }

    #[must_use]
    pub fn unit(&self, id: UnitId) -> Option<&Unit> {
        let index =
            id.0.checked_sub(1)
                .and_then(|value| usize::try_from(value).ok())?;
        self.units.get(index).filter(|unit| unit.id == id)
    }

    #[must_use]
    pub fn enemy(&self, id: EnemyId) -> Option<&Enemy> {
        let index =
            id.0.checked_sub(1)
                .and_then(|value| usize::try_from(value).ok())?;
        self.enemies.get(index).filter(|enemy| enemy.id == id)
    }

    #[must_use]
    pub fn select_units(&self, rect: WorldRect) -> Vec<UnitId> {
        self.units
            .iter()
            .filter(|unit| rect.contains(unit.position))
            .map(|unit| unit.id)
            .collect()
    }

    /// Issues a move order for the selected units.
    ///
    /// # Errors
    ///
    /// Returns [`CommandRejection`] when the selection is empty, the destination is invalid,
    /// no path exists, or one of the unit ids is unknown.
    pub fn move_units(
        &mut self,
        units: &[UnitId],
        destination: WorldPoint,
    ) -> Result<(), CommandRejection> {
        if units.is_empty() {
            return Err(CommandRejection::EmptySelection);
        }

        if !self.contains_world_point(destination) {
            return Err(CommandRejection::DestinationOutOfBounds);
        }

        if self.is_world_point_blocked(destination) {
            return Err(CommandRejection::BlockedDestination);
        }

        let unit_count = units.len();
        let mut orders = Vec::with_capacity(unit_count);
        for (index, unit_id) in units.iter().copied().enumerate() {
            let target = group_move_target(
                index,
                unit_count,
                destination,
                self.grid_size,
                GROUP_MOVE_SPACING,
            );
            let Some(unit) = self.unit(unit_id) else {
                return Err(CommandRejection::UnknownUnit(unit_id));
            };
            let Some(path) = find_path(unit.position, target, &self.terrain, self.grid_size) else {
                return Err(CommandRejection::NoPath);
            };
            orders.push((unit_id, path));
        }

        for (unit_id, path) in orders {
            let Some(unit) = self.unit_mut(unit_id) else {
                return Err(CommandRejection::UnknownUnit(unit_id));
            };
            unit.path = path;
        }

        Ok(())
    }

    pub fn spawn_horde(&mut self, enemy_count: usize) {
        let Some(target) = self.command_center_target() else {
            return;
        };

        self.enemies.reserve(enemy_count);
        for _ in 0..enemy_count {
            let spawn_index =
                usize::try_from(self.next_enemy_id.saturating_sub(1)).unwrap_or(usize::MAX);
            let id = EnemyId::new(self.next_enemy_id);
            self.next_enemy_id = self.next_enemy_id.saturating_add(1);

            let position = horde_spawn_position(spawn_index, self.grid_size);
            let path = find_path(position, target, &self.terrain, self.grid_size);

            self.enemies.push(Enemy {
                id,
                position,
                path: path.unwrap_or_default(),
            });
        }
    }

    pub fn step(&mut self, steps: u32) {
        for _ in 0..steps {
            self.tick = self.tick.saturating_add(1);
            for unit in &mut self.units {
                move_unit_toward_target(unit, &self.terrain, self.grid_size);
            }
            for enemy in &mut self.enemies {
                move_enemy_toward_target(enemy, &self.terrain, self.grid_size);
            }
        }
    }

    pub fn set_worker_count(&mut self, worker_count: usize) {
        self.units.clear();

        let width = usize::from(self.grid_size.width);
        let height = usize::from(self.grid_size.height);
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
                position: WorldPoint::new(f32::from(x) + jitter_x, f32::from(y) + jitter_y),
                path: Vec::new(),
            });
        }
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
                position: WorldPoint::new(
                    f32::from(center.x) + x_offset,
                    f32::from(center.y) + y_offset,
                ),
                path: Vec::new(),
            });
        }
    }

    fn contains_world_point(&self, point: WorldPoint) -> bool {
        point.x >= 0.0
            && point.y >= 0.0
            && point.x < f32::from(self.grid_size.width)
            && point.y < f32::from(self.grid_size.height)
    }

    fn is_world_point_blocked(&self, point: WorldPoint) -> bool {
        self.world_point_to_grid(point)
            .and_then(|coord| self.terrain_at(coord))
            == Some(TerrainCell::Blocked)
    }

    fn world_point_to_grid(&self, point: WorldPoint) -> Option<GridCoord> {
        world_point_to_grid(point, self.grid_size)
    }

    fn terrain_index(&self, coord: GridCoord) -> Option<usize> {
        terrain_index(coord, self.grid_size)
    }

    fn grid_coord_from_index(&self, index: usize) -> Option<GridCoord> {
        let width = usize::from(self.grid_size.width);
        if width == 0 || index >= self.terrain.len() {
            return None;
        }

        Some(GridCoord::new(
            u16::try_from(index % width).ok()?,
            u16::try_from(index / width).ok()?,
        ))
    }

    fn command_center_target(&self) -> Option<WorldPoint> {
        self.buildings
            .first()
            .map(|building| grid_cell_center(building.position))
    }

    fn unit_mut(&mut self, id: UnitId) -> Option<&mut Unit> {
        let index =
            id.0.checked_sub(1)
                .and_then(|value| usize::try_from(value).ok())?;
        self.units.get_mut(index).filter(|unit| unit.id == id)
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
    move_agent_toward_target(
        &mut unit.position,
        &mut unit.path,
        WORKER_SPEED_PER_TICK,
        terrain,
        grid_size,
    );
}

fn move_enemy_toward_target(enemy: &mut Enemy, terrain: &[TerrainCell], grid_size: GridSize) {
    move_agent_toward_target(
        &mut enemy.position,
        &mut enemy.path,
        ENEMY_SPEED_PER_TICK,
        terrain,
        grid_size,
    );
}

fn move_agent_toward_target(
    position: &mut WorldPoint,
    path: &mut Vec<WorldPoint>,
    speed_per_tick: f32,
    terrain: &[TerrainCell],
    grid_size: GridSize,
) {
    let Some(next_target) = path.last().copied() else {
        return;
    };

    let dx = next_target.x - position.x;
    let dy = next_target.y - position.y;
    let distance = dx.hypot(dy);

    if distance <= speed_per_tick {
        if is_point_blocked(next_target, terrain, grid_size) {
            path.clear();
            return;
        }

        *position = next_target;
        path.pop();
        return;
    }

    let scale = speed_per_tick / distance;
    let next_position = WorldPoint::new(position.x + dx * scale, position.y + dy * scale);
    if is_point_blocked(next_position, terrain, grid_size) {
        path.clear();
        return;
    }

    *position = next_position;
}

fn find_path(
    start: WorldPoint,
    destination: WorldPoint,
    terrain: &[TerrainCell],
    grid_size: GridSize,
) -> Option<Vec<WorldPoint>> {
    let start_point = start;
    let start = world_point_to_grid(start, grid_size)?;
    let goal = world_point_to_grid(destination, grid_size)?;

    if is_coord_blocked(goal, terrain, grid_size) {
        return None;
    }

    if start == goal {
        return Some(vec![destination]);
    }

    if straight_path_is_clear(start_point, destination, terrain, grid_size) {
        return Some(vec![destination]);
    }

    let start_index = terrain_index(start, grid_size)?;
    let goal_index = terrain_index(goal, grid_size)?;
    let mut frontier = VecDeque::from([start]);
    let mut came_from = vec![None; grid_size.cell_count()];
    came_from[start_index] = Some(start);

    while let Some(current) = frontier.pop_front() {
        if current == goal {
            break;
        }

        for (x_delta, y_delta) in PATH_NEIGHBORS {
            let Some(neighbor) = neighbor_coord(current, x_delta, y_delta, grid_size) else {
                continue;
            };
            if is_coord_blocked(neighbor, terrain, grid_size) {
                continue;
            }

            let neighbor_index = terrain_index(neighbor, grid_size)?;
            if came_from[neighbor_index].is_some() {
                continue;
            }

            came_from[neighbor_index] = Some(current);
            frontier.push_back(neighbor);
        }
    }

    came_from[goal_index]?;

    let mut cells = Vec::new();
    let mut current = goal;
    while current != start {
        cells.push(current);
        let current_index = terrain_index(current, grid_size)?;
        current = came_from[current_index]?;
    }
    Some(
        cells
            .into_iter()
            .map(|coord| {
                if coord == goal {
                    destination
                } else {
                    grid_cell_center(coord)
                }
            })
            .collect(),
    )
}

fn neighbor_coord(
    coord: GridCoord,
    x_delta: i16,
    y_delta: i16,
    grid_size: GridSize,
) -> Option<GridCoord> {
    let x = i32::from(coord.x) + i32::from(x_delta);
    let y = i32::from(coord.y) + i32::from(y_delta);
    if x < 0 || y < 0 || x >= i32::from(grid_size.width) || y >= i32::from(grid_size.height) {
        return None;
    }

    Some(GridCoord::new(
        u16::try_from(x).ok()?,
        u16::try_from(y).ok()?,
    ))
}

fn grid_cell_center(coord: GridCoord) -> WorldPoint {
    WorldPoint::new(f32::from(coord.x) + 0.5, f32::from(coord.y) + 0.5)
}

fn horde_spawn_position(index: usize, grid_size: GridSize) -> WorldPoint {
    let lap = index / 4;
    match index % 4 {
        0 => WorldPoint::new(0.5, edge_lane(lap, grid_size.height)),
        1 => WorldPoint::new(edge_max(grid_size.width), edge_lane(lap, grid_size.height)),
        2 => WorldPoint::new(edge_lane(lap, grid_size.width), 0.5),
        _ => WorldPoint::new(edge_lane(lap, grid_size.width), edge_max(grid_size.height)),
    }
}

fn edge_lane(lap: usize, length: u16) -> f32 {
    if length <= 2 {
        return 0.5;
    }

    let inner_length = usize::from(length) - 2;
    let lane = 1 + (lap * 3) % inner_length;
    f32::from(u16::try_from(lane).unwrap_or(1)) + 0.5
}

fn edge_max(length: u16) -> f32 {
    (f32::from(length) - 0.5).max(0.5)
}

fn straight_path_is_clear(
    start: WorldPoint,
    destination: WorldPoint,
    terrain: &[TerrainCell],
    grid_size: GridSize,
) -> bool {
    let longest_axis = (destination.x - start.x)
        .abs()
        .max((destination.y - start.y).abs());
    let sample_count = bounded_floor_to_u16((longest_axis * 4.0).ceil()).max(1);

    for sample in 1..=sample_count {
        let t = f32::from(sample) / f32::from(sample_count);
        let point = WorldPoint::new(
            start.x + (destination.x - start.x) * t,
            start.y + (destination.y - start.y) * t,
        );
        if is_point_blocked(point, terrain, grid_size) {
            return false;
        }
    }

    true
}

fn is_point_blocked(point: WorldPoint, terrain: &[TerrainCell], grid_size: GridSize) -> bool {
    let Some(coord) = world_point_to_grid(point, grid_size) else {
        return false;
    };

    is_coord_blocked(coord, terrain, grid_size)
}

fn is_coord_blocked(coord: GridCoord, terrain: &[TerrainCell], grid_size: GridSize) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_world() -> GameWorld {
        GameWorld::new(GridSize::new(32, 24))
    }

    #[test]
    fn bootstrap_world_contains_command_center_and_workers() {
        let world = test_world();

        assert_eq!(world.grid_size(), GridSize::new(32, 24));
        assert_eq!(world.tick(), 0);
        assert_eq!(world.units().len(), 6);
        assert_eq!(world.buildings().len(), 1);
        assert_eq!(world.buildings()[0].position, GridCoord::new(16, 12));
    }

    #[test]
    fn accepted_move_advances_units_on_fixed_tick() {
        let mut world = test_world();
        let unit_id = world.units()[0].id;
        let start = world.units()[0].position;

        let result = world.move_units(&[unit_id], WorldPoint::new(start.x + 3.0, start.y));
        world.step(1);

        let unit = world.unit(unit_id).expect("unit should still exist");
        assert_eq!(result, Ok(()));
        assert_eq!(world.tick(), 1);
        assert!(unit.position.x > start.x);
        assert!((unit.position.y - start.y).abs() < f32::EPSILON);
    }

    #[test]
    fn multi_unit_move_assigns_spaced_formation_targets() {
        let mut world = test_world();
        let units = world
            .units()
            .iter()
            .take(4)
            .map(|unit| unit.id)
            .collect::<Vec<_>>();

        let result = world.move_units(&units, WorldPoint::new(16.0, 12.0));

        assert_eq!(result, Ok(()));
        let targets = units
            .iter()
            .map(|unit_id| {
                world
                    .unit(*unit_id)
                    .and_then(|unit| unit.path.last().copied())
                    .unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(targets[0], WorldPoint::new(15.6, 11.6));
        assert_eq!(targets[1], WorldPoint::new(16.4, 11.6));
        assert_eq!(targets[2], WorldPoint::new(15.6, 12.4));
        assert_eq!(targets[3], WorldPoint::new(16.4, 12.4));
    }

    #[test]
    fn selection_uses_world_space_rectangle_without_bevy_types() {
        let world = test_world();

        let selected = world.select_units(WorldRect::from_corners(
            WorldPoint::new(13.0, 9.0),
            WorldPoint::new(19.0, 15.0),
        ));

        assert_eq!(selected.len(), 6);
    }

    #[test]
    fn terrain_cells_can_be_blocked_and_counted() {
        let mut world = test_world();
        let coord = GridCoord::new(4, 5);

        assert_eq!(world.terrain_at(coord), Some(TerrainCell::Clear));
        assert_eq!(world.blocked_cell_count(), 0);

        assert_eq!(world.set_terrain(coord, TerrainCell::Blocked), Ok(()));

        assert_eq!(world.terrain_at(coord), Some(TerrainCell::Blocked));
        assert_eq!(world.blocked_cell_count(), 1);
        assert_eq!(world.blocked_cells().collect::<Vec<_>>(), vec![coord]);
    }

    #[test]
    fn move_rejects_blocked_destination_without_mutating_world() {
        let mut world = test_world();
        let unit_id = world.units()[0].id;

        assert_eq!(
            world.set_terrain(GridCoord::new(18, 12), TerrainCell::Blocked),
            Ok(())
        );
        let before = world.clone();
        let result = world.move_units(&[unit_id], WorldPoint::new(18.2, 12.2));

        assert_eq!(result, Err(CommandRejection::BlockedDestination));
        assert_eq!(world, before);
    }

    #[test]
    fn unit_paths_around_blocked_cell() {
        let mut world = test_world();
        let unit_id = world.units()[0].id;
        let start = world.units()[0].position;
        let blocked = GridCoord::new(
            bounded_floor_to_u16(start.x + 1.0),
            bounded_floor_to_u16(start.y),
        );

        assert_eq!(world.set_terrain(blocked, TerrainCell::Blocked), Ok(()));
        assert_eq!(
            world.move_units(&[unit_id], WorldPoint::new(start.x + 3.2, start.y + 0.2)),
            Ok(())
        );
        world.step(20);

        let unit = world.unit(unit_id).expect("unit should still exist");
        assert!((unit.position.x - (start.x + 3.2)).abs() < 0.01);
        assert!((unit.position.y - (start.y + 0.2)).abs() < 0.01);
        assert!(unit.path.is_empty());
    }

    #[test]
    fn move_rejects_unreachable_destination_without_mutating_world() {
        let mut world = test_world();
        let unit_id = world.units()[0].id;
        let destination = WorldPoint::new(18.2, 12.2);

        for coord in [
            GridCoord::new(17, 12),
            GridCoord::new(19, 12),
            GridCoord::new(18, 11),
            GridCoord::new(18, 13),
        ] {
            assert_eq!(world.set_terrain(coord, TerrainCell::Blocked), Ok(()));
        }

        let before = world.clone();
        let result = world.move_units(&[unit_id], destination);

        assert_eq!(result, Err(CommandRejection::NoPath));
        assert_eq!(world, before);
    }

    #[test]
    fn horde_spawns_from_map_edges_and_paths_to_command_center() {
        let mut world = test_world();

        world.spawn_horde(8);

        assert_eq!(world.enemies().len(), 8);
        for enemy in world.enemies() {
            assert!(is_edge_position(enemy.position, world.grid_size()));
            assert!(enemy.path.contains(&WorldPoint::new(16.5, 12.5)));
        }
    }

    #[test]
    fn horde_moves_toward_command_center_on_fixed_tick() {
        let mut world = test_world();
        world.spawn_horde(1);
        let enemy_id = world.enemies()[0].id;
        let target = WorldPoint::new(16.5, 12.5);
        let start = world.enemies()[0].position;

        world.step(3);

        let enemy = world.enemy(enemy_id).expect("enemy should still exist");
        assert!(distance(enemy.position, target) < distance(start, target));
    }

    #[test]
    fn horde_simulation_is_deterministic() {
        let mut first = test_world();
        let mut second = test_world();

        first.spawn_horde(32);
        second.spawn_horde(32);
        first.step(8);
        second.step(8);

        assert_eq!(first, second);
    }

    #[test]
    fn rejected_command_does_not_mutate_state() {
        let mut world = test_world();
        let before = world.clone();

        let result = world.move_units(&[UnitId::new(404)], WorldPoint::new(4.0, 4.0));

        assert_eq!(result, Err(CommandRejection::UnknownUnit(UnitId::new(404))));
        assert_eq!(world, before);
    }

    #[test]
    fn same_commands_produce_same_world() {
        let mut first = test_world();
        let mut second = test_world();
        let unit_id = first.units()[0].id;
        let destination = WorldPoint::new(18.0, 12.0);

        assert_eq!(unit_id, second.units()[0].id);
        assert_eq!(first.move_units(&[unit_id], destination), Ok(()));
        assert_eq!(second.move_units(&[unit_id], destination), Ok(()));

        first.step(4);
        second.step(4);

        assert_eq!(first, second);
    }

    #[test]
    fn stress_population_sets_exact_worker_count() {
        let mut world = test_world();

        world.set_worker_count(1_000);

        assert_eq!(world.units().len(), 1_000);
        assert_eq!(world.unit(UnitId::new(1_000)), world.units().last());
    }

    #[test]
    fn stress_population_is_deterministic_and_in_bounds() {
        let mut first = test_world();
        let mut second = test_world();

        first.set_worker_count(5_000);
        second.set_worker_count(5_000);

        assert_eq!(first, second);
        for (index, unit) in first.units().iter().enumerate() {
            assert_eq!(unit.id, UnitId::new(u32::try_from(index + 1).unwrap()));
            assert!(unit.position.x >= 0.0);
            assert!(unit.position.y >= 0.0);
            assert!(unit.position.x < f32::from(first.grid_size().width));
            assert!(unit.position.y < f32::from(first.grid_size().height));
        }
    }

    fn is_edge_position(position: WorldPoint, grid_size: GridSize) -> bool {
        position.x < 1.0
            || position.y < 1.0
            || position.x > f32::from(grid_size.width) - 1.0
            || position.y > f32::from(grid_size.height) - 1.0
    }

    fn distance(first: WorldPoint, second: WorldPoint) -> f32 {
        (first.x - second.x).hypot(first.y - second.y)
    }
}
