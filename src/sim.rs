use std::collections::VecDeque;

const STARTING_WORKERS: usize = 6;
const GROUP_MOVE_SPACING: f32 = 0.8;
const SPATIAL_CHUNK_SIZE: u16 = 16;
const SIM_SECONDS_PER_TICK: f32 = 0.1;
const PATH_NEIGHBORS: [(i16, i16); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];
const PLAYER_SPAWN_OFFSETS: [(f32, f32); 12] = [
    (2.0, 0.0),
    (2.0, 1.0),
    (1.0, 2.0),
    (0.0, 2.0),
    (-1.0, 2.0),
    (-2.0, 1.0),
    (-2.0, 0.0),
    (-2.0, -1.0),
    (-1.0, -2.0),
    (0.0, -2.0),
    (1.0, -2.0),
    (2.0, -1.0),
];

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AgentStats {
    pub max_health: f32,
    pub armor: f32,
    pub attack_damage: f32,
    pub attack_range: f32,
    pub attacks_per_second: f32,
    pub movement_speed_per_tick: f32,
    pub vision_range: f32,
    pub noise: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnitKind {
    Worker,
    Ranger,
    Soldier,
}

impl UnitKind {
    #[must_use]
    pub const fn stats(self) -> AgentStats {
        match self {
            Self::Worker => AgentStats {
                max_health: 60.0,
                armor: 0.0,
                attack_damage: 5.0,
                attack_range: 3.0,
                attacks_per_second: 1.0,
                movement_speed_per_tick: 0.10,
                vision_range: 6.0,
                noise: 1.0,
            },
            Self::Ranger => AgentStats {
                max_health: 60.0,
                armor: 0.05,
                attack_damage: 10.0,
                attack_range: 6.0,
                attacks_per_second: 1.0,
                movement_speed_per_tick: 0.16,
                vision_range: 8.0,
                noise: 1.0,
            },
            Self::Soldier => AgentStats {
                max_health: 120.0,
                armor: 0.4,
                attack_damage: 16.0,
                attack_range: 5.0,
                attacks_per_second: 2.0,
                movement_speed_per_tick: 0.12,
                vision_range: 6.0,
                noise: 3.0,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Unit {
    pub id: UnitId,
    pub kind: UnitKind,
    pub health: f32,
    pub position: WorldPoint,
    attack_cooldown_ticks: u16,
    path: Vec<WorldPoint>,
}

impl Unit {
    #[must_use]
    pub fn new(id: UnitId, kind: UnitKind, position: WorldPoint) -> Self {
        Self {
            id,
            kind,
            health: kind.stats().max_health,
            position,
            attack_cooldown_ticks: 0,
            path: Vec::new(),
        }
    }

    #[must_use]
    pub const fn stats(&self) -> AgentStats {
        self.kind.stats()
    }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnemyKind {
    InfectedDecrepit,
}

impl EnemyKind {
    #[must_use]
    pub const fn stats(self) -> AgentStats {
        match self {
            Self::InfectedDecrepit => AgentStats {
                max_health: 35.0,
                armor: 0.0,
                attack_damage: 5.0,
                attack_range: 0.8,
                attacks_per_second: 0.8,
                movement_speed_per_tick: 0.06,
                vision_range: 4.0,
                noise: 0.0,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Enemy {
    pub id: EnemyId,
    pub kind: EnemyKind,
    pub health: f32,
    pub position: WorldPoint,
    attack_cooldown_ticks: u16,
    path: Vec<WorldPoint>,
}

impl Enemy {
    #[must_use]
    pub fn new(id: EnemyId, kind: EnemyKind, position: WorldPoint) -> Self {
        Self {
            id,
            kind,
            health: kind.stats().max_health,
            position,
            attack_cooldown_ticks: 0,
            path: Vec::new(),
        }
    }

    #[must_use]
    pub const fn stats(&self) -> AgentStats {
        self.kind.stats()
    }
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
struct SpatialIndex<Id> {
    grid_size: GridSize,
    chunk_size: u16,
    chunk_columns: usize,
    chunk_rows: usize,
    buckets: Vec<Vec<Id>>,
}

impl<Id: Copy> SpatialIndex<Id> {
    fn new(grid_size: GridSize, chunk_size: u16) -> Self {
        let chunk_size = chunk_size.max(1);
        let chunk_size_usize = usize::from(chunk_size);
        let chunk_columns = usize::from(grid_size.width).div_ceil(chunk_size_usize);
        let chunk_rows = usize::from(grid_size.height).div_ceil(chunk_size_usize);
        let bucket_count = chunk_columns.saturating_mul(chunk_rows);

        Self {
            grid_size,
            chunk_size,
            chunk_columns,
            chunk_rows,
            buckets: vec![Vec::new(); bucket_count],
        }
    }

    fn rebuild(&mut self, entities: impl IntoIterator<Item = (Id, WorldPoint)>) {
        for bucket in &mut self.buckets {
            bucket.clear();
        }

        for (id, position) in entities {
            let Some(index) = self.bucket_index_for_point(position) else {
                continue;
            };
            self.buckets[index].push(id);
        }
    }

    #[must_use]
    fn ids_in_rect(&self, rect: WorldRect) -> Vec<Id> {
        let Some((min_x, max_x, min_y, max_y)) = self.chunk_range_for_rect(rect) else {
            return Vec::new();
        };

        let mut ids = Vec::new();
        for chunk_y in min_y..=max_y {
            for chunk_x in min_x..=max_x {
                let Some(index) = self.bucket_index_for_chunk(chunk_x, chunk_y) else {
                    continue;
                };
                ids.extend(self.buckets[index].iter().copied());
            }
        }
        ids
    }

    fn bucket_index_for_point(&self, point: WorldPoint) -> Option<usize> {
        let coord = world_point_to_grid(point, self.grid_size)?;
        let chunk_x = usize::from(coord.x) / usize::from(self.chunk_size);
        let chunk_y = usize::from(coord.y) / usize::from(self.chunk_size);
        self.bucket_index_for_chunk(chunk_x, chunk_y)
    }

    fn bucket_index_for_chunk(&self, chunk_x: usize, chunk_y: usize) -> Option<usize> {
        if chunk_x >= self.chunk_columns || chunk_y >= self.chunk_rows {
            return None;
        }

        Some(chunk_y * self.chunk_columns + chunk_x)
    }

    fn chunk_range_for_rect(&self, rect: WorldRect) -> Option<(usize, usize, usize, usize)> {
        let (min_cell_x, max_cell_x) =
            clamped_cell_range(rect.min.x, rect.max.x, self.grid_size.width)?;
        let (min_cell_y, max_cell_y) =
            clamped_cell_range(rect.min.y, rect.max.y, self.grid_size.height)?;
        let chunk_size = usize::from(self.chunk_size);

        Some((
            usize::from(min_cell_x) / chunk_size,
            usize::from(max_cell_x) / chunk_size,
            usize::from(min_cell_y) / chunk_size,
            usize::from(max_cell_y) / chunk_size,
        ))
    }
}

#[derive(Clone, Debug, PartialEq)]
struct FlowField {
    grid_size: GridSize,
    destination: GridCoord,
    next_steps: Vec<Option<GridCoord>>,
}

impl FlowField {
    fn build(
        destination: WorldPoint,
        terrain: &[TerrainCell],
        grid_size: GridSize,
    ) -> Option<Self> {
        let destination_coord = world_point_to_grid(destination, grid_size)?;
        if is_coord_blocked(destination_coord, terrain, grid_size) {
            return None;
        }

        let destination_index = terrain_index(destination_coord, grid_size)?;
        let mut next_steps = vec![None; grid_size.cell_count()];
        let mut frontier = VecDeque::from([destination_coord]);
        next_steps[destination_index] = Some(destination_coord);

        while let Some(current) = frontier.pop_front() {
            for (x_delta, y_delta) in PATH_NEIGHBORS {
                let Some(neighbor) = neighbor_coord(current, x_delta, y_delta, grid_size) else {
                    continue;
                };
                if is_coord_blocked(neighbor, terrain, grid_size) {
                    continue;
                }

                let neighbor_index = terrain_index(neighbor, grid_size)?;
                if next_steps[neighbor_index].is_some() {
                    continue;
                }

                next_steps[neighbor_index] = Some(current);
                frontier.push_back(neighbor);
            }
        }

        Some(Self {
            grid_size,
            destination: destination_coord,
            next_steps,
        })
    }

    fn path_from(
        &self,
        start: WorldPoint,
        destination: WorldPoint,
        terrain: &[TerrainCell],
    ) -> Option<Vec<WorldPoint>> {
        let start_coord = world_point_to_grid(start, self.grid_size)?;
        if is_coord_blocked(start_coord, terrain, self.grid_size) {
            return None;
        }

        if start_coord == self.destination {
            return Some(vec![destination]);
        }

        if straight_path_is_clear(start, destination, terrain, self.grid_size) {
            return Some(vec![destination]);
        }

        let mut current = start_coord;
        let mut path = Vec::new();
        for _ in 0..self.grid_size.cell_count() {
            let current_index = terrain_index(current, self.grid_size)?;
            let next = self.next_steps[current_index]?;

            if next == current {
                break;
            }

            path.push(if next == self.destination {
                destination
            } else {
                grid_cell_center(next)
            });
            current = next;

            if current == self.destination {
                break;
            }
        }

        if current != self.destination {
            return None;
        }

        path.reverse();
        Some(path)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GameWorld {
    grid_size: GridSize,
    tick: u64,
    units: Vec<Unit>,
    enemies: Vec<Enemy>,
    next_unit_id: u32,
    next_enemy_id: u32,
    buildings: Vec<Building>,
    terrain: Vec<TerrainCell>,
    unit_chunks: SpatialIndex<UnitId>,
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
            next_unit_id: 1,
            next_enemy_id: 1,
            buildings: vec![Building { position: center }],
            terrain: vec![TerrainCell::Clear; grid_size.cell_count()],
            unit_chunks: SpatialIndex::new(grid_size, SPATIAL_CHUNK_SIZE),
        };

        world.spawn_starting_workers(center);
        world.next_unit_id = u32::try_from(STARTING_WORKERS + 1).unwrap_or(u32::MAX);
        world.rebuild_unit_spatial_index();
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
        self.units.iter().find(|unit| unit.id == id)
    }

    #[must_use]
    pub fn enemy(&self, id: EnemyId) -> Option<&Enemy> {
        self.enemies.iter().find(|enemy| enemy.id == id)
    }

    #[must_use]
    pub fn select_units(&self, rect: WorldRect) -> Vec<UnitId> {
        self.unit_chunks
            .ids_in_rect(rect)
            .into_iter()
            .filter(|unit_id| {
                self.unit(*unit_id)
                    .is_some_and(|unit| rect.contains(unit.position))
            })
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
        let flow_field = FlowField::build(destination, &self.terrain, self.grid_size)
            .ok_or(CommandRejection::NoPath)?;
        let mut selected_units = Vec::with_capacity(unit_count);
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
            selected_units.push((unit_id, unit.position, target));
        }

        let mut orders = Vec::with_capacity(unit_count);
        for (unit_id, position, target) in selected_units {
            let Some(mut path) = flow_field.path_from(position, destination, &self.terrain) else {
                return Err(CommandRejection::NoPath);
            };
            append_formation_target(
                &mut path,
                destination,
                target,
                &self.terrain,
                self.grid_size,
            )
            .ok_or(CommandRejection::NoPath)?;
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

            let mut enemy = Enemy::new(id, EnemyKind::InfectedDecrepit, position);
            enemy.path = path.unwrap_or_default();
            self.enemies.push(enemy);
        }
    }

    #[must_use]
    pub fn spawn_unit_near_command_center(&mut self, kind: UnitKind) -> Option<UnitId> {
        let center = self.command_center_target()?;
        let position =
            player_spawn_position(center, self.units.len(), self.grid_size, &self.terrain)?;
        let id = UnitId::new(self.next_unit_id);
        self.next_unit_id = self.next_unit_id.saturating_add(1);
        self.units.push(Unit::new(id, kind, position));
        self.rebuild_unit_spatial_index();
        Some(id)
    }

    pub fn step(&mut self, steps: u32) {
        for _ in 0..steps {
            self.tick = self.tick.saturating_add(1);
            self.resolve_combat();
            for unit in &mut self.units {
                move_unit_toward_target(unit, &self.terrain, self.grid_size);
            }
            for enemy in &mut self.enemies {
                move_enemy_toward_target(enemy, &self.terrain, self.grid_size);
            }
            self.rebuild_unit_spatial_index();
        }
    }

    pub fn set_worker_count(&mut self, worker_count: usize) {
        self.units.clear();
        self.next_unit_id = u32::try_from(worker_count.saturating_add(1)).unwrap_or(u32::MAX);

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

            self.units.push(Unit::new(
                UnitId::new(u32::try_from(index + 1).unwrap_or(u32::MAX)),
                UnitKind::Worker,
                WorldPoint::new(f32::from(x) + jitter_x, f32::from(y) + jitter_y),
            ));
        }
        self.rebuild_unit_spatial_index();
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
            self.units.push(Unit::new(
                UnitId::new(u32::try_from(index + 1).expect("starting worker id fits in u32")),
                UnitKind::Worker,
                WorldPoint::new(
                    f32::from(center.x) + x_offset,
                    f32::from(center.y) + y_offset,
                ),
            ));
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
        self.units.iter_mut().find(|unit| unit.id == id)
    }

    fn rebuild_unit_spatial_index(&mut self) {
        self.unit_chunks
            .rebuild(self.units.iter().map(|unit| (unit.id, unit.position)));
    }

    fn resolve_combat(&mut self) {
        tick_unit_cooldowns(&mut self.units);
        tick_enemy_cooldowns(&mut self.enemies);

        let mut enemy_damage = vec![0.0; self.enemies.len()];
        for unit in &mut self.units {
            if unit.attack_cooldown_ticks > 0 {
                continue;
            }
            let Some(enemy_index) = nearest_enemy_in_range(unit, &self.enemies) else {
                continue;
            };

            let enemy = &self.enemies[enemy_index];
            enemy_damage[enemy_index] +=
                armor_reduced_damage(unit.stats().attack_damage, enemy.stats().armor);
            unit.attack_cooldown_ticks = attack_cooldown_ticks(unit.stats());
        }

        let mut unit_damage = vec![0.0; self.units.len()];
        for enemy in &mut self.enemies {
            if enemy.attack_cooldown_ticks > 0 {
                continue;
            }
            let Some(unit_index) = nearest_unit_in_range(enemy, &self.units) else {
                continue;
            };

            let unit = &self.units[unit_index];
            unit_damage[unit_index] +=
                armor_reduced_damage(enemy.stats().attack_damage, unit.stats().armor);
            enemy.attack_cooldown_ticks = attack_cooldown_ticks(enemy.stats());
        }

        for (enemy, damage) in self.enemies.iter_mut().zip(enemy_damage) {
            enemy.health -= damage;
        }
        for (unit, damage) in self.units.iter_mut().zip(unit_damage) {
            unit.health -= damage;
        }

        self.enemies.retain(|enemy| enemy.health > 0.0);
        self.units.retain(|unit| unit.health > 0.0);
    }
}

fn tick_unit_cooldowns(units: &mut [Unit]) {
    for unit in units {
        unit.attack_cooldown_ticks = unit.attack_cooldown_ticks.saturating_sub(1);
    }
}

fn tick_enemy_cooldowns(enemies: &mut [Enemy]) {
    for enemy in enemies {
        enemy.attack_cooldown_ticks = enemy.attack_cooldown_ticks.saturating_sub(1);
    }
}

fn nearest_enemy_in_range(unit: &Unit, enemies: &[Enemy]) -> Option<usize> {
    nearest_target_in_range(
        unit.position,
        unit.stats().attack_range,
        enemies.iter().map(|enemy| enemy.position),
    )
}

fn nearest_unit_in_range(enemy: &Enemy, units: &[Unit]) -> Option<usize> {
    nearest_target_in_range(
        enemy.position,
        enemy.stats().attack_range,
        units.iter().map(|unit| unit.position),
    )
}

fn nearest_target_in_range(
    attacker_position: WorldPoint,
    attack_range: f32,
    targets: impl Iterator<Item = WorldPoint>,
) -> Option<usize> {
    targets
        .enumerate()
        .filter_map(|(index, position)| {
            let distance = distance(attacker_position, position);
            (distance <= attack_range).then_some((index, distance))
        })
        .min_by(|first, second| first.1.total_cmp(&second.1))
        .map(|(index, _)| index)
}

fn attack_cooldown_ticks(stats: AgentStats) -> u16 {
    if stats.attacks_per_second <= 0.0 {
        return u16::MAX;
    }

    let ticks = (1.0 / stats.attacks_per_second / SIM_SECONDS_PER_TICK).ceil();
    bounded_floor_to_u16(ticks.max(1.0))
}

fn armor_reduced_damage(damage: f32, armor: f32) -> f32 {
    damage * (1.0 - armor).clamp(0.0, 1.0)
}

fn player_spawn_position(
    center: WorldPoint,
    spawn_index: usize,
    grid_size: GridSize,
    terrain: &[TerrainCell],
) -> Option<WorldPoint> {
    for offset_index in 0..PLAYER_SPAWN_OFFSETS.len() {
        let (x_offset, y_offset) =
            PLAYER_SPAWN_OFFSETS[(spawn_index + offset_index) % PLAYER_SPAWN_OFFSETS.len()];
        let position = WorldPoint::new(center.x + x_offset, center.y + y_offset);
        if contains_world_point(position, grid_size)
            && !is_point_blocked(position, terrain, grid_size)
        {
            return Some(position);
        }
    }

    if contains_world_point(center, grid_size) && !is_point_blocked(center, terrain, grid_size) {
        return Some(center);
    }

    None
}

fn contains_world_point(point: WorldPoint, grid_size: GridSize) -> bool {
    point.x >= 0.0
        && point.y >= 0.0
        && point.x < f32::from(grid_size.width)
        && point.y < f32::from(grid_size.height)
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

fn append_formation_target(
    path: &mut Vec<WorldPoint>,
    destination: WorldPoint,
    target: WorldPoint,
    terrain: &[TerrainCell],
    grid_size: GridSize,
) -> Option<()> {
    if target == destination {
        return Some(());
    }

    if is_point_blocked(target, terrain, grid_size)
        || !straight_path_is_clear(destination, target, terrain, grid_size)
    {
        return None;
    }

    path.insert(0, target);
    Some(())
}

fn move_unit_toward_target(unit: &mut Unit, terrain: &[TerrainCell], grid_size: GridSize) {
    let speed = unit.stats().movement_speed_per_tick;
    move_agent_toward_target(
        &mut unit.position,
        &mut unit.path,
        speed,
        terrain,
        grid_size,
    );
}

fn move_enemy_toward_target(enemy: &mut Enemy, terrain: &[TerrainCell], grid_size: GridSize) {
    let speed = enemy.stats().movement_speed_per_tick;
    move_agent_toward_target(
        &mut enemy.position,
        &mut enemy.path,
        speed,
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
    let distance = world_distance(dx, dy);

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

fn world_distance(x_delta: f32, y_delta: f32) -> f32 {
    x_delta.mul_add(x_delta, y_delta * y_delta).sqrt()
}

fn find_path(
    start: WorldPoint,
    destination: WorldPoint,
    terrain: &[TerrainCell],
    grid_size: GridSize,
) -> Option<Vec<WorldPoint>> {
    let flow_field = FlowField::build(destination, terrain, grid_size)?;
    flow_field.path_from(start, destination, terrain)
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

fn distance(first: WorldPoint, second: WorldPoint) -> f32 {
    world_distance(first.x - second.x, first.y - second.y)
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

fn clamped_cell_range(min: f32, max: f32, limit: u16) -> Option<(u16, u16)> {
    if limit == 0 || max < 0.0 || min >= f32::from(limit) {
        return None;
    }

    let max_cell = limit.saturating_sub(1);
    let min_cell = bounded_floor_to_u16(min.max(0.0)).min(max_cell);
    let max_cell = bounded_floor_to_u16(max.min(f32::from(max_cell))).min(max_cell);

    Some((min_cell, max_cell))
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
    fn starting_workers_spawn_with_worker_stats() {
        let world = test_world();

        for unit in world.units() {
            assert_eq!(unit.kind, UnitKind::Worker);
            assert_approx_eq(unit.health, UnitKind::Worker.stats().max_health);
            assert_eq!(unit.stats(), UnitKind::Worker.stats());
            assert_approx_eq(unit.stats().movement_speed_per_tick, 0.10);
            assert!(unit.stats().attack_damage > 0.0);
            assert!(unit.stats().attack_range > 0.0);
        }
    }

    #[test]
    fn player_units_can_be_spawned_near_the_command_center() {
        let mut world = test_world();

        let spawned_id = world
            .spawn_unit_near_command_center(UnitKind::Ranger)
            .expect("spawn near command center should succeed");

        let spawned = world.unit(spawned_id).expect("spawned unit should exist");
        assert_eq!(world.units().len(), STARTING_WORKERS + 1);
        assert_eq!(spawned.id, UnitId::new(7));
        assert_eq!(spawned.kind, UnitKind::Ranger);
        assert_approx_eq(spawned.health, UnitKind::Ranger.stats().max_health);
        assert!(distance(spawned.position, WorldPoint::new(16.5, 12.5)) <= 3.0);
    }

    #[test]
    fn units_attack_enemies_in_range_and_remove_dead_enemies() {
        let mut world = GameWorld::new(GridSize::new(8, 8));
        world.units = vec![Unit::new(
            UnitId::new(1),
            UnitKind::Soldier,
            WorldPoint::new(3.5, 3.5),
        )];
        world.enemies = vec![Enemy::new(
            EnemyId::new(1),
            EnemyKind::InfectedDecrepit,
            WorldPoint::new(4.2, 3.5),
        )];

        world.step(12);

        assert!(world.enemies().is_empty());
        assert_eq!(world.units().len(), 1);
        assert!(world.unit(UnitId::new(1)).unwrap().health < UnitKind::Soldier.stats().max_health);
    }

    #[test]
    fn enemies_attack_units_in_range_and_remove_dead_units() {
        let mut world = GameWorld::new(GridSize::new(8, 8));
        let mut worker = Unit::new(UnitId::new(1), UnitKind::Worker, WorldPoint::new(3.5, 3.5));
        worker.health = 5.0;
        world.units = vec![worker];
        world.enemies = vec![Enemy::new(
            EnemyId::new(1),
            EnemyKind::InfectedDecrepit,
            WorldPoint::new(4.0, 3.5),
        )];

        world.step(1);

        assert!(world.units().is_empty());
        assert_eq!(world.enemies().len(), 1);
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
        let destination = WorldPoint::new(16.0, 12.0);
        let paths = units
            .iter()
            .map(|unit_id| world.unit(*unit_id).map(|unit| unit.path.clone()).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(paths[0].first().copied(), Some(WorldPoint::new(15.6, 11.6)));
        assert_eq!(paths[1].first().copied(), Some(WorldPoint::new(16.4, 11.6)));
        assert_eq!(paths[2].first().copied(), Some(WorldPoint::new(15.6, 12.4)));
        assert_eq!(paths[3].first().copied(), Some(WorldPoint::new(16.4, 12.4)));
        for path in paths {
            assert!(path.contains(&destination));
        }
    }

    #[test]
    fn selection_uses_spatial_chunks_to_reduce_candidate_units() {
        let mut world = test_world();
        world.set_worker_count(5_000);
        let rect =
            WorldRect::from_corners(WorldPoint::new(10.0, 10.0), WorldPoint::new(12.0, 12.0));

        let candidates = world.unit_chunks.ids_in_rect(rect);
        let selected = world.select_units(rect);

        assert!(!selected.is_empty());
        assert!(candidates.len() < world.units().len());
        assert!(selected.iter().all(|unit_id| candidates.contains(unit_id)));
    }

    #[test]
    fn selection_uses_world_space_rectangle() {
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
        world.step(60);

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
            assert_eq!(enemy.kind, EnemyKind::InfectedDecrepit);
            assert_approx_eq(enemy.health, EnemyKind::InfectedDecrepit.stats().max_health);
            assert_eq!(enemy.stats(), EnemyKind::InfectedDecrepit.stats());
            assert_approx_eq(enemy.stats().movement_speed_per_tick, 0.06);
            assert!(enemy.stats().attack_damage > 0.0);
            assert!(is_edge_position(enemy.position, world.grid_size()));
            assert!(enemy.path.contains(&WorldPoint::new(16.5, 12.5)));
        }
    }

    #[test]
    fn movement_uses_unit_and_enemy_stats() {
        let terrain = vec![TerrainCell::Clear; GridSize::new(8, 1).cell_count()];
        let mut unit = Unit {
            id: UnitId::new(1),
            kind: UnitKind::Worker,
            health: UnitKind::Worker.stats().max_health,
            position: WorldPoint::new(0.5, 0.5),
            attack_cooldown_ticks: 0,
            path: vec![WorldPoint::new(4.5, 0.5)],
        };
        let mut enemy = Enemy {
            id: EnemyId::new(1),
            kind: EnemyKind::InfectedDecrepit,
            health: EnemyKind::InfectedDecrepit.stats().max_health,
            position: WorldPoint::new(0.5, 0.5),
            attack_cooldown_ticks: 0,
            path: vec![WorldPoint::new(4.5, 0.5)],
        };

        move_unit_toward_target(&mut unit, &terrain, GridSize::new(8, 1));
        move_enemy_toward_target(&mut enemy, &terrain, GridSize::new(8, 1));

        assert!((unit.position.x - 0.6).abs() < f32::EPSILON);
        assert!((enemy.position.x - 0.56).abs() < f32::EPSILON);
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
        world_distance(first.x - second.x, first.y - second.y)
    }

    fn assert_approx_eq(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < f32::EPSILON,
            "expected {actual} to equal {expected}"
        );
    }
}
