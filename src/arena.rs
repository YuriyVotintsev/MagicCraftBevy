use bevy::prelude::*;

pub const WINDOW_WIDTH: f32 = 1280.0;
pub const WINDOW_HEIGHT: f32 = 720.0;
pub const ARENA_WIDTH: f32 = 1920.0;
pub const ARENA_HEIGHT: f32 = 1080.0;
pub const BORDER_THICKNESS: f32 = 10.0;

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_camera, spawn_arena))
            .add_systems(PostUpdate, camera_follow);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_arena(mut commands: Commands) {
    let half_width = ARENA_WIDTH / 2.0;
    let half_height = ARENA_HEIGHT / 2.0;
    let border_color = Color::srgb(0.8, 0.8, 0.8);

    commands
        .spawn(Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(
                ARENA_WIDTH + BORDER_THICKNESS * 2.0,
                BORDER_THICKNESS,
            )),
            ..default()
        })
        .insert(Transform::from_xyz(
            0.0,
            half_height + BORDER_THICKNESS / 2.0,
            0.0,
        ));

    commands
        .spawn(Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(
                ARENA_WIDTH + BORDER_THICKNESS * 2.0,
                BORDER_THICKNESS,
            )),
            ..default()
        })
        .insert(Transform::from_xyz(
            0.0,
            -half_height - BORDER_THICKNESS / 2.0,
            0.0,
        ));

    commands
        .spawn(Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(
                BORDER_THICKNESS,
                ARENA_HEIGHT + BORDER_THICKNESS * 2.0,
            )),
            ..default()
        })
        .insert(Transform::from_xyz(
            -half_width - BORDER_THICKNESS / 2.0,
            0.0,
            0.0,
        ));

    commands
        .spawn(Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(
                BORDER_THICKNESS,
                ARENA_HEIGHT + BORDER_THICKNESS * 2.0,
            )),
            ..default()
        })
        .insert(Transform::from_xyz(
            half_width + BORDER_THICKNESS / 2.0,
            0.0,
            0.0,
        ));
}

fn camera_follow(
    player_query: Query<&Transform, With<crate::player::Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<crate::player::Player>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y;
}
