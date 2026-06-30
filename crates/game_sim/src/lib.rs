const STARTING_WORKERS: usize = 6;
const WORKER_SPEED_PER_TICK: f32 = 1.5;

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
    UnknownUnit(UnitId),
}

#[derive(Clone, Debug)]
pub struct GameWorld {
    config: SimConfig,
    seed: u64,
    tick: u64,
    units: Vec<Unit>,
    buildings: Vec<Building>,
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

                if let Some(unknown_id) = units.iter().copied().find(|id| self.unit(*id).is_none())
                {
                    return CommandResult::Rejected(CommandRejection::UnknownUnit(unknown_id));
                }

                for unit_id in units {
                    let Some(unit) = self.units.iter_mut().find(|unit| unit.id == unit_id) else {
                        return CommandResult::Rejected(CommandRejection::UnknownUnit(unit_id));
                    };
                    unit.target = Some(destination);
                }

                CommandResult::Accepted
            }
        }
    }

    pub fn step(&mut self, steps: u32) {
        for _ in 0..steps {
            self.tick = self.tick.saturating_add(1);
            for unit in &mut self.units {
                move_unit_toward_target(unit);
            }
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
}

fn move_unit_toward_target(unit: &mut Unit) {
    let Some(target) = unit.target else {
        return;
    };

    let dx = target.x - unit.position.x;
    let dy = target.y - unit.position.y;
    let distance = dx.hypot(dy);

    if distance <= WORKER_SPEED_PER_TICK {
        unit.position = target;
        unit.target = None;
        return;
    }

    let scale = WORKER_SPEED_PER_TICK / distance;
    unit.position.x += dx * scale;
    unit.position.y += dy * scale;
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
    fn selection_uses_world_space_rectangle_without_bevy_types() {
        let world = GameWorld::new(SimConfig::new(32, 24), 7);

        let selected = world.select_units(WorldRect::from_corners(
            WorldPoint::new(13.0, 9.0),
            WorldPoint::new(19.0, 15.0),
        ));

        assert_eq!(selected.len(), 6);
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
}
