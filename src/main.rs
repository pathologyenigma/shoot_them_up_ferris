// rust way to use the code from others
use bevy::prelude::*;
const WINDOW_SIZE: (f32, f32) = (600., 800.);
const UNIT_SIZE: (f32, f32) = (60., 45.);
const PLAYER_SPAWN_POINT: (f32, f32) = (0., -400. + UNIT_SIZE.1 / 2.);
const PLAYER_MOVE_SPEED: f32 = 3.5;
#[derive(Component)]
struct Player;
fn main() {
    App::new()
    .insert_resource(WindowDescriptor{
        title: "shoot them up ferris".to_string(),
        width: WINDOW_SIZE.0,
        height: WINDOW_SIZE.1,
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .add_startup_system(start_up_system)
    .add_system(move_player)
    .run();
}
fn start_up_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(SpriteBundle{
        texture: asset_server.load::<Image, &str>("textures/ferris.png"),
        sprite: Sprite {
            custom_size: Some(Vec2::new(UNIT_SIZE.0, UNIT_SIZE.1)),
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(PLAYER_SPAWN_POINT.0, PLAYER_SPAWN_POINT.1, 0.),
            ..Default::default()
        },
        ..Default::default()
    }).insert(Player);
}
fn move_player(keys: Res<Input<KeyCode>>, mut player: Query<&mut Transform, With<Player>>) {
    let mut direction = Vec2::default();
    if keys.any_pressed([KeyCode::W]) {
        direction.y += 1.;
    }
    if keys.any_pressed([KeyCode::S]) {
        direction.y += -1.;
    }
    if keys.any_pressed([KeyCode::D]) {
        direction.x += 1.;
    }
    if keys.any_pressed([KeyCode::A]) {
        direction.x += -1.;
    }
    if direction == Vec2::ZERO {
        return;
    }
    let move_delta = (direction * PLAYER_MOVE_SPEED).extend(0.);
    for mut transform in player.iter_mut() {
        transform.translation += move_delta;
    }
}