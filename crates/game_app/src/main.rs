#![allow(clippy::needless_pass_by_value)]

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use game_sim::{
    Command, CommandResult, GameWorld, GridSize, SimConfig, UnitId, WorldPoint, WorldRect,
};

const TILE_SIZE: f32 = 32.0;
const SIM_TICK_SECONDS: f32 = 0.1;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.04, 0.05, 0.045)))
        .insert_resource(SimResource {
            world: GameWorld::new(SimConfig::new(32, 24), 7),
            accumulator: 0.0,
        })
        .insert_resource(Selection::default())
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Ashen Frontier".to_string(),
                    resolution: (1280, 720).into(),
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
                selection_input,
                move_input,
                tick_simulation,
                sync_unit_visuals,
                update_fps_text,
                draw_world,
            ),
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

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct UnitVisual {
    id: UnitId,
}

#[derive(Component)]
struct BuildingVisual;

#[derive(Component)]
struct FpsText;

fn setup(mut commands: Commands, sim: Res<SimResource>) {
    commands.spawn((Camera2d, MainCamera));
    let grid_size = sim.world.grid_size();

    commands.spawn((
        Text::new(format_fps(None)),
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
        FpsText,
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
        commands.spawn((
            Sprite::from_color(Color::srgb(0.76, 0.82, 0.64), Vec2::splat(TILE_SIZE * 0.45)),
            Transform::from_translation(world_to_translation(unit.position, grid_size, 2.0)),
            UnitVisual { id: unit.id },
        ));
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

fn selection_input(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    sim: Res<SimResource>,
    mut selection: ResMut<Selection>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(world_position) = cursor_world_position(&windows, &camera) else {
        return;
    };
    let sim_position = bevy_to_world_point(world_position, sim.world.grid_size());
    let rect = WorldRect::from_corners(
        WorldPoint::new(sim_position.x - 0.75, sim_position.y - 0.75),
        WorldPoint::new(sim_position.x + 0.75, sim_position.y + 0.75),
    );

    selection.units = sim.world.select_units(rect);
}

fn move_input(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut sim: ResMut<SimResource>,
    selection: Res<Selection>,
) {
    if !buttons.just_pressed(MouseButton::Right) || selection.units.is_empty() {
        return;
    }

    let Some(world_position) = cursor_world_position(&windows, &camera) else {
        return;
    };
    let destination = bevy_to_world_point(world_position, sim.world.grid_size());
    let result = sim.world.submit_command(Command::MoveUnits {
        units: selection.units.clone(),
        destination,
    });

    if result != CommandResult::Accepted {
        eprintln!("move command rejected: {result:?}");
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
    sim: Res<SimResource>,
    selection: Res<Selection>,
    mut units: Query<(&UnitVisual, &mut Transform, &mut Sprite)>,
) {
    let grid_size = sim.world.grid_size();

    for (visual, mut transform, mut sprite) in &mut units {
        let Some(unit) = sim.world.unit(visual.id) else {
            continue;
        };

        transform.translation = world_to_translation(unit.position, grid_size, 2.0);
        sprite.color = if selection.units.contains(&visual.id) {
            Color::srgb(0.95, 0.86, 0.34)
        } else {
            Color::srgb(0.76, 0.82, 0.64)
        };
    }
}

fn update_fps_text(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<FpsText>>) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(bevy::diagnostic::Diagnostic::smoothed);

    for mut text in &mut query {
        text.0 = format_fps(fps);
    }
}

fn draw_world(mut gizmos: Gizmos, sim: Res<SimResource>, selection: Res<Selection>) {
    let grid = sim.world.grid_size();
    let width = f32::from(grid.width) * TILE_SIZE;
    let height = f32::from(grid.height) * TILE_SIZE;
    let origin = Vec2::new(-width / 2.0, -height / 2.0);
    let grid_color = Color::srgba(0.42, 0.50, 0.44, 0.2);

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
}

fn cursor_world_position(
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera: &Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Option<Vec2> {
    let window = windows.iter().next()?;
    let cursor_position = window.cursor_position()?;
    let (camera, camera_transform) = camera.iter().next()?;

    camera
        .viewport_to_world_2d(camera_transform, cursor_position)
        .ok()
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

fn format_fps(fps: Option<f64>) -> String {
    fps.map_or_else(|| "FPS: --".to_string(), |fps| format!("FPS: {fps:.1}"))
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
}
