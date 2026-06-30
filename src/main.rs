#![allow(clippy::needless_pass_by_value)]

use std::collections::HashSet;

use ashen_frontier::sim::{
    Enemy, EnemyId, GameWorld, GridSize, TerrainCell, Unit, UnitId, WorldPoint, WorldRect,
};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow};

const GRID_WIDTH: u16 = 32;
const GRID_HEIGHT: u16 = 24;
const TILE_SIZE: f32 = 32.0;
const SIM_TICK_SECONDS: f32 = 0.1;
const CLICK_SELECT_RADIUS: f32 = 0.75;
const DRAG_SELECT_THRESHOLD_PIXELS: f32 = 6.0;
const STRESS_PRESET_SMALL: usize = 100;
const STRESS_PRESET_MEDIUM: usize = 1_000;
const STRESS_PRESET_LARGE: usize = 5_000;
const HORDE_PRESET_COUNT: usize = 64;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.04, 0.05, 0.045)))
        .insert_resource(SimResource {
            world: new_world(),
            accumulator: 0.0,
        })
        .insert_resource(Selection::default())
        .insert_resource(DragSelection::default())
        .insert_resource(InteractionMode::default())
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Ashen Frontier".to_string(),
                    resolution: (1280, 720).into(),
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                camera_controls,
                toggle_interaction_mode,
                paint_terrain_input,
                selection_input,
                move_input,
                apply_stress_hotkeys,
                tick_simulation,
                sync_unit_visuals,
                sync_enemy_visuals,
                update_hud_text,
                draw_world,
            )
                .chain(),
        )
        .run();
}

#[derive(Resource)]
struct SimResource {
    world: GameWorld,
    accumulator: f32,
}

#[derive(Resource, Default)]
struct Selection {
    units: Vec<UnitId>,
}

#[derive(Resource, Default)]
struct DragSelection {
    start_world: Option<WorldPoint>,
    current_world: Option<WorldPoint>,
    start_screen: Option<Vec2>,
    current_screen: Option<Vec2>,
}

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
enum InteractionMode {
    #[default]
    Select,
    PaintBlocked,
}

impl InteractionMode {
    fn label(self) -> &'static str {
        match self {
            Self::Select => "select",
            Self::PaintBlocked => "paint blocked",
        }
    }
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct UnitVisual {
    id: UnitId,
}

#[derive(Component)]
struct EnemyVisual {
    id: EnemyId,
}

#[derive(Component)]
struct BuildingVisual;

#[derive(Component)]
struct HudText;

#[derive(Clone, Copy)]
struct HudStats {
    fps: Option<f64>,
    unit_count: usize,
    enemy_count: usize,
    selected_count: usize,
    tick: u64,
    blocked_cell_count: usize,
    mode: InteractionMode,
}

fn setup(mut commands: Commands, sim: Res<SimResource>) {
    commands.spawn((Camera2d, MainCamera));
    let grid_size = sim.world.grid_size();

    commands.spawn((
        Text::new(format_hud(HudStats {
            fps: None,
            unit_count: sim.world.units().len(),
            enemy_count: sim.world.enemies().len(),
            selected_count: 0,
            tick: sim.world.tick(),
            blocked_cell_count: sim.world.blocked_cell_count(),
            mode: InteractionMode::Select,
        })),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        TextColor(Color::srgb(0.86, 0.91, 0.82)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            top: Val::Px(10.0),
            ..default()
        },
        HudText,
    ));

    for building in sim.world.buildings() {
        commands.spawn((
            Sprite::from_color(Color::srgb(0.18, 0.38, 0.48), Vec2::splat(TILE_SIZE * 1.6)),
            Transform::from_translation(grid_to_translation(
                f32::from(building.position.x),
                f32::from(building.position.y),
                grid_size,
                1.0,
            )),
            BuildingVisual,
        ));
    }

    for unit in sim.world.units() {
        spawn_unit_visual(&mut commands, unit, grid_size);
    }

    for enemy in sim.world.enemies() {
        spawn_enemy_visual(&mut commands, enemy, grid_size);
    }
}

fn camera_controls(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut camera: Query<&mut Transform, With<MainCamera>>,
) {
    let Some(mut transform) = camera.iter_mut().next() else {
        return;
    };

    let mut movement = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        movement.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        movement.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        movement.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        movement.x += 1.0;
    }

    if movement.length_squared() > 0.0 {
        transform.translation += movement.normalize() * 520.0 * time.delta_secs();
    }

    let mut zoom_delta = 0.0;
    if keys.pressed(KeyCode::KeyQ) {
        zoom_delta += 1.0;
    }
    if keys.pressed(KeyCode::KeyE) {
        zoom_delta -= 1.0;
    }

    if zoom_delta != 0.0 {
        let zoom = 1.0 + zoom_delta * time.delta_secs();
        transform.scale = (transform.scale * zoom).clamp(Vec3::splat(0.55), Vec3::splat(2.25));
    }
}

fn toggle_interaction_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut interaction: ResMut<InteractionMode>,
) {
    if !keys.just_pressed(KeyCode::KeyB) {
        return;
    }

    *interaction = match *interaction {
        InteractionMode::Select => InteractionMode::PaintBlocked,
        InteractionMode::PaintBlocked => InteractionMode::Select,
    };
}

fn paint_terrain_input(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    interaction: Res<InteractionMode>,
    mut sim: ResMut<SimResource>,
) {
    if *interaction != InteractionMode::PaintBlocked || !buttons.pressed(MouseButton::Left) {
        return;
    }

    let Some((_, world_position)) = cursor_screen_and_world_position(&windows, &camera) else {
        return;
    };
    let sim_position = bevy_to_world_point(world_position, sim.world.grid_size());
    let _ = sim
        .world
        .set_terrain_at_point(sim_position, TerrainCell::Blocked);
}

fn selection_input(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    sim: Res<SimResource>,
    interaction: Res<InteractionMode>,
    mut drag: ResMut<DragSelection>,
    mut selection: ResMut<Selection>,
) {
    if *interaction != InteractionMode::Select {
        return;
    }

    if buttons.just_pressed(MouseButton::Left) {
        let Some((screen_position, world_position)) =
            cursor_screen_and_world_position(&windows, &camera)
        else {
            return;
        };
        let sim_position = bevy_to_world_point(world_position, sim.world.grid_size());
        drag.start_world = Some(sim_position);
        drag.current_world = Some(sim_position);
        drag.start_screen = Some(screen_position);
        drag.current_screen = Some(screen_position);
    }

    if buttons.pressed(MouseButton::Left)
        && let Some((screen_position, world_position)) =
            cursor_screen_and_world_position(&windows, &camera)
    {
        drag.current_world = Some(bevy_to_world_point(world_position, sim.world.grid_size()));
        drag.current_screen = Some(screen_position);
    }

    if buttons.just_released(MouseButton::Left) {
        if let Some((screen_position, world_position)) =
            cursor_screen_and_world_position(&windows, &camera)
        {
            drag.current_world = Some(bevy_to_world_point(world_position, sim.world.grid_size()));
            drag.current_screen = Some(screen_position);
        }

        if let (Some(start_world), Some(current_world), Some(start_screen), Some(current_screen)) = (
            drag.start_world,
            drag.current_world,
            drag.start_screen,
            drag.current_screen,
        ) {
            let rect =
                selection_rect_from_drag(start_world, current_world, start_screen, current_screen);
            selection.units = sim.world.select_units(rect);
        }

        *drag = DragSelection::default();
    }
}

fn move_input(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut sim: ResMut<SimResource>,
    selection: Res<Selection>,
    interaction: Res<InteractionMode>,
) {
    if *interaction != InteractionMode::Select
        || !buttons.just_pressed(MouseButton::Right)
        || selection.units.is_empty()
    {
        return;
    }

    let Some((_, world_position)) = cursor_screen_and_world_position(&windows, &camera) else {
        return;
    };
    let destination = bevy_to_world_point(world_position, sim.world.grid_size());
    let result = sim.world.move_units(&selection.units, destination);

    if let Err(result) = result {
        eprintln!("move command rejected: {result:?}");
    }
}

fn apply_stress_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    mut sim: ResMut<SimResource>,
    mut selection: ResMut<Selection>,
) {
    let worker_count = if keys.just_pressed(KeyCode::Digit1) {
        Some(STRESS_PRESET_SMALL)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(STRESS_PRESET_MEDIUM)
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(STRESS_PRESET_LARGE)
    } else {
        None
    };

    if let Some(worker_count) = worker_count {
        sim.world.set_worker_count(worker_count);
        selection.units.clear();
    }

    if keys.just_pressed(KeyCode::KeyH) {
        sim.world.spawn_horde(HORDE_PRESET_COUNT);
    }

    if keys.just_pressed(KeyCode::KeyR) {
        sim.world = new_world();
        sim.accumulator = 0.0;
        selection.units.clear();
    }
}

fn tick_simulation(time: Res<Time>, mut sim: ResMut<SimResource>) {
    sim.accumulator += time.delta_secs();
    while sim.accumulator >= SIM_TICK_SECONDS {
        sim.world.step(1);
        sim.accumulator -= SIM_TICK_SECONDS;
    }
}

fn sync_unit_visuals(
    mut commands: Commands,
    sim: Res<SimResource>,
    selection: Res<Selection>,
    mut units: Query<(Entity, &UnitVisual, &mut Transform, &mut Sprite)>,
) {
    let grid_size = sim.world.grid_size();
    let simulated_ids = sim
        .world
        .units()
        .iter()
        .map(|unit| unit.id)
        .collect::<HashSet<_>>();
    let selected_ids = selection.units.iter().copied().collect::<HashSet<_>>();
    let mut visual_ids = HashSet::with_capacity(simulated_ids.len());

    for (entity, visual, mut transform, mut sprite) in &mut units {
        if !simulated_ids.contains(&visual.id) {
            commands.entity(entity).despawn();
            continue;
        }

        visual_ids.insert(visual.id);
        let Some(unit) = sim.world.unit(visual.id) else {
            continue;
        };

        transform.translation = world_to_translation(unit.position, grid_size, 2.0);
        sprite.color = if selected_ids.contains(&visual.id) {
            Color::srgb(0.95, 0.86, 0.34)
        } else {
            Color::srgb(0.76, 0.82, 0.64)
        };
    }

    for unit in sim.world.units() {
        if !visual_ids.contains(&unit.id) {
            spawn_unit_visual(&mut commands, unit, grid_size);
        }
    }
}

fn sync_enemy_visuals(
    mut commands: Commands,
    sim: Res<SimResource>,
    mut enemies: Query<(Entity, &EnemyVisual, &mut Transform)>,
) {
    let grid_size = sim.world.grid_size();
    let simulated_ids = sim
        .world
        .enemies()
        .iter()
        .map(|enemy| enemy.id)
        .collect::<HashSet<_>>();
    let mut visual_ids = HashSet::with_capacity(simulated_ids.len());

    for (entity, visual, mut transform) in &mut enemies {
        if !simulated_ids.contains(&visual.id) {
            commands.entity(entity).despawn();
            continue;
        }

        visual_ids.insert(visual.id);
        let Some(enemy) = sim.world.enemy(visual.id) else {
            continue;
        };

        transform.translation = world_to_translation(enemy.position, grid_size, 2.1);
    }

    for enemy in sim.world.enemies() {
        if !visual_ids.contains(&enemy.id) {
            spawn_enemy_visual(&mut commands, enemy, grid_size);
        }
    }
}

fn update_hud_text(
    diagnostics: Res<DiagnosticsStore>,
    sim: Res<SimResource>,
    selection: Res<Selection>,
    interaction: Res<InteractionMode>,
    mut query: Query<&mut Text, With<HudText>>,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(bevy::diagnostic::Diagnostic::smoothed);
    let stats = HudStats {
        fps,
        unit_count: sim.world.units().len(),
        enemy_count: sim.world.enemies().len(),
        selected_count: selection.units.len(),
        tick: sim.world.tick(),
        blocked_cell_count: sim.world.blocked_cell_count(),
        mode: *interaction,
    };

    for mut text in &mut query {
        text.0 = format_hud(stats);
    }
}

fn draw_world(
    mut gizmos: Gizmos,
    sim: Res<SimResource>,
    selection: Res<Selection>,
    drag: Res<DragSelection>,
) {
    let grid = sim.world.grid_size();
    let width = f32::from(grid.width) * TILE_SIZE;
    let height = f32::from(grid.height) * TILE_SIZE;
    let origin = Vec2::new(-width / 2.0, -height / 2.0);
    let grid_color = Color::srgba(0.42, 0.50, 0.44, 0.2);

    for blocked in sim.world.blocked_cells() {
        let center =
            grid_to_translation(f32::from(blocked.x), f32::from(blocked.y), grid, 0.5).truncate();
        draw_blocked_cell(&mut gizmos, center);
    }

    for x in 0..=grid.width {
        let x_position = origin.x + f32::from(x) * TILE_SIZE;
        gizmos.line_2d(
            Vec2::new(x_position, origin.y),
            Vec2::new(x_position, origin.y + height),
            grid_color,
        );
    }

    for y in 0..=grid.height {
        let y_position = origin.y + f32::from(y) * TILE_SIZE;
        gizmos.line_2d(
            Vec2::new(origin.x, y_position),
            Vec2::new(origin.x + width, y_position),
            grid_color,
        );
    }

    for selected in &selection.units {
        let Some(unit) = sim.world.unit(*selected) else {
            continue;
        };
        gizmos.circle_2d(
            Isometry2d::from_translation(world_to_translation(unit.position, grid, 3.0).truncate()),
            TILE_SIZE * 0.34,
            Color::srgb(0.95, 0.86, 0.34),
        );
    }

    if let (Some(start_world), Some(current_world), Some(start_screen), Some(current_screen)) = (
        drag.start_world,
        drag.current_world,
        drag.start_screen,
        drag.current_screen,
    ) && is_drag_selection(start_screen, current_screen)
    {
        draw_selection_rect(&mut gizmos, start_world, current_world, grid);
    }
}

fn cursor_screen_and_world_position(
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera: &Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Option<(Vec2, Vec2)> {
    let window = windows.iter().next()?;
    let cursor_position = window.cursor_position()?;
    let (camera, camera_transform) = camera.iter().next()?;
    let world_position = camera
        .viewport_to_world_2d(camera_transform, cursor_position)
        .ok()?;

    Some((cursor_position, world_position))
}

fn world_to_translation(position: WorldPoint, grid_size: GridSize, z: f32) -> Vec3 {
    grid_to_translation(position.x, position.y, grid_size, z)
}

fn grid_to_translation(x: f32, y: f32, grid_size: GridSize, z: f32) -> Vec3 {
    Vec3::new(
        (x - f32::from(grid_size.width) / 2.0) * TILE_SIZE + TILE_SIZE / 2.0,
        (y - f32::from(grid_size.height) / 2.0) * TILE_SIZE + TILE_SIZE / 2.0,
        z,
    )
}

fn bevy_to_world_point(position: Vec2, grid_size: GridSize) -> WorldPoint {
    WorldPoint::new(
        (position.x - TILE_SIZE / 2.0) / TILE_SIZE + f32::from(grid_size.width) / 2.0,
        (position.y - TILE_SIZE / 2.0) / TILE_SIZE + f32::from(grid_size.height) / 2.0,
    )
}

fn new_world() -> GameWorld {
    GameWorld::new(GridSize::new(GRID_WIDTH, GRID_HEIGHT))
}

fn spawn_unit_visual(commands: &mut Commands, unit: &Unit, grid_size: GridSize) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.76, 0.82, 0.64), Vec2::splat(TILE_SIZE * 0.45)),
        Transform::from_translation(world_to_translation(unit.position, grid_size, 2.0)),
        UnitVisual { id: unit.id },
    ));
}

fn spawn_enemy_visual(commands: &mut Commands, enemy: &Enemy, grid_size: GridSize) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.78, 0.18, 0.16), Vec2::splat(TILE_SIZE * 0.38)),
        Transform::from_translation(world_to_translation(enemy.position, grid_size, 2.1)),
        EnemyVisual { id: enemy.id },
    ));
}

fn selection_rect_from_drag(
    start_world: WorldPoint,
    current_world: WorldPoint,
    start_screen: Vec2,
    current_screen: Vec2,
) -> WorldRect {
    if is_drag_selection(start_screen, current_screen) {
        WorldRect::from_corners(start_world, current_world)
    } else {
        WorldRect::from_corners(
            WorldPoint::new(
                start_world.x - CLICK_SELECT_RADIUS,
                start_world.y - CLICK_SELECT_RADIUS,
            ),
            WorldPoint::new(
                start_world.x + CLICK_SELECT_RADIUS,
                start_world.y + CLICK_SELECT_RADIUS,
            ),
        )
    }
}

fn is_drag_selection(start_screen: Vec2, current_screen: Vec2) -> bool {
    start_screen.distance(current_screen) > DRAG_SELECT_THRESHOLD_PIXELS
}

fn draw_selection_rect(
    gizmos: &mut Gizmos,
    start_world: WorldPoint,
    current_world: WorldPoint,
    grid_size: GridSize,
) {
    let start = world_to_translation(start_world, grid_size, 4.0).truncate();
    let end = world_to_translation(current_world, grid_size, 4.0).truncate();
    let top_left = Vec2::new(start.x.min(end.x), start.y.max(end.y));
    let top_right = Vec2::new(start.x.max(end.x), start.y.max(end.y));
    let bottom_right = Vec2::new(start.x.max(end.x), start.y.min(end.y));
    let bottom_left = Vec2::new(start.x.min(end.x), start.y.min(end.y));
    let color = Color::srgb(0.95, 0.86, 0.34);

    gizmos.line_2d(top_left, top_right, color);
    gizmos.line_2d(top_right, bottom_right, color);
    gizmos.line_2d(bottom_right, bottom_left, color);
    gizmos.line_2d(bottom_left, top_left, color);
}

fn draw_blocked_cell(gizmos: &mut Gizmos, center: Vec2) {
    let size = Vec2::splat(TILE_SIZE * 0.92);
    let color = Color::srgb(0.78, 0.22, 0.18);
    let half = size / 2.0;

    gizmos.rect_2d(Isometry2d::from_translation(center), size, color);
    gizmos.line_2d(center - half, center + half, color);
    gizmos.line_2d(
        Vec2::new(center.x - half.x, center.y + half.y),
        Vec2::new(center.x + half.x, center.y - half.y),
        color,
    );
}

fn format_fps(fps: Option<f64>) -> String {
    fps.map_or_else(|| "FPS: --".to_string(), |fps| format!("FPS: {fps:.1}"))
}

fn format_hud(stats: HudStats) -> String {
    format!(
        "{}\nUnits: {}\nEnemies: {}\nSelected: {}\nTick: {}\nBlocked: {}\nMode: {}\nVSync: off\nHotkeys: B=mode H=horde 1=100 2=1000 3=5000 R=reset",
        format_fps(stats.fps),
        stats.unit_count,
        stats.enemy_count,
        stats.selected_count,
        stats.tick,
        stats.blocked_cell_count,
        stats.mode.label()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fps_label_formats_missing_diagnostic() {
        assert_eq!(format_fps(None), "FPS: --");
    }

    #[test]
    fn fps_label_formats_smoothed_value() {
        assert_eq!(format_fps(Some(59.94)), "FPS: 59.9");
    }

    #[test]
    fn hud_label_includes_performance_and_stress_stats() {
        let stats = HudStats {
            fps: Some(144.46),
            unit_count: 1_000,
            enemy_count: 64,
            selected_count: 12,
            tick: 42,
            blocked_cell_count: 8,
            mode: InteractionMode::PaintBlocked,
        };

        assert_eq!(
            format_hud(stats),
            "FPS: 144.5\nUnits: 1000\nEnemies: 64\nSelected: 12\nTick: 42\nBlocked: 8\nMode: paint blocked\nVSync: off\nHotkeys: B=mode H=horde 1=100 2=1000 3=5000 R=reset"
        );
    }

    #[test]
    fn selection_rect_uses_click_radius_for_tiny_drags() {
        let rect = selection_rect_from_drag(
            WorldPoint::new(10.0, 10.0),
            WorldPoint::new(10.1, 10.1),
            Vec2::new(100.0, 100.0),
            Vec2::new(102.0, 101.0),
        );

        assert!(rect.contains(WorldPoint::new(9.3, 10.0)));
        assert!(!rect.contains(WorldPoint::new(9.2, 10.0)));
    }

    #[test]
    fn selection_rect_uses_drag_corners_for_large_drags() {
        let rect = selection_rect_from_drag(
            WorldPoint::new(8.0, 6.0),
            WorldPoint::new(12.0, 9.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(180.0, 140.0),
        );

        assert!(rect.contains(WorldPoint::new(10.0, 8.0)));
        assert!(!rect.contains(WorldPoint::new(7.9, 8.0)));
    }
}
