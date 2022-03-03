use std::collections::VecDeque;

// rust way to use the code from others
use bevy::prelude::*;
const WINDOW_SIZE: (f32, f32) = (600., 800.);
const UNIT_SIZE: (f32, f32) = (60., 45.);
const PLAYER_SPAWN_POINT: (f32, f32) = (0., -400. + UNIT_SIZE.1 / 2.);
const PLAYER_MOVE_SPEED: f32 = 3.5;
const ENEMY_SIZE: (f32, f32) = (60., 60.);
const ENEMY_SPAWN_Y: f32 = 400. - ENEMY_SIZE.1;
const ENEMY_X_RANGE: std::ops::Range<f32> = (-300. + ENEMY_SIZE.0 / 2.)..(300. - ENEMY_SIZE.0 / 2.);
const PLAYER_BULLET_Y: f32 = 37.;
const PLAYER_WEAPON_COOL_DOWN: f32 = 0.3;
use rand::prelude::*;
#[derive(Component)]
struct Player {
    bullets: BulletPool,
    weapon_cool_down: Timer,
}
#[derive(Component)]
struct AutoMove(bool);
#[derive(Component)]
struct AutoShoot(bool);
#[derive(Bundle)]
struct Enemy {
    auto_move: AutoMove,
    auto_shoot: AutoShoot,
}
struct EnemyPool(VecDeque<Entity>);
impl EnemyPool {
    pub fn init(commands: &mut Commands, asset_server: &Res<AssetServer>, size: usize) {
        let mut pool = VecDeque::new();
        for _ in 0..size {
            let entity = commands
                .spawn_bundle(SpriteBundle {
                    texture: asset_server.load("textures/golang.png"),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(ENEMY_SIZE.0, ENEMY_SIZE.1)),
                        ..Default::default()
                    },
                    // transform: Transform {
                    //     translation: Vec3::new(x, y, 0.),
                    //     ..Default::default()
                    // },
                    visibility: Visibility { is_visible: false },
                    ..Default::default()
                })
                .insert_bundle(Enemy {
                    auto_move: AutoMove(false),
                    auto_shoot: AutoShoot(false),
                })
                .id();
            pool.push_back(entity);
        }
        commands.insert_resource(Self(pool));
    }
    pub fn spawn(&mut self) -> Option<Entity> {
        self.0.pop_front()
    }
    pub fn recycle(&mut self, entity: Entity) {
        self.0.push_back(entity);
    }
}
struct BulletPool(VecDeque<Entity>);
impl BulletPool {
    pub fn init(
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        size: usize,
        for_player: bool,
    ) -> Self {
        let mut pool = VecDeque::new();
        let texture = if for_player {
            asset_server.load("textures/bullet.png")
        } else {
            asset_server.load("textures/enemy_bullet.png")
        };
        for _ in 0..size {
            let entity = commands
                .spawn_bundle(SpriteBundle {
                    texture: texture.clone(),
                    visibility: Visibility { is_visible: false },
                    ..Default::default()
                })
                .insert(AutoMove(true))
                .id();
            pool.push_back(entity);
        }
        Self(pool)
    }
    pub fn spawn(&mut self) -> Option<Entity> {
        self.0.pop_front()
    }
    pub fn recycle(&mut self, entity: Entity) {
        self.0.push_back(entity);
    }
}
fn main() {
    let timer = Timer::from_seconds(3., false);
    App::new()
        .insert_resource(WindowDescriptor {
            title: "shoot them up ferris".to_string(),
            width: WINDOW_SIZE.0,
            height: WINDOW_SIZE.1,
            ..Default::default()
        })
        .insert_resource(timer)
        .add_plugins(DefaultPlugins)
        .add_startup_system(start_up_system)
        .add_system(move_player)
        .add_system(spawn_enemy)
        .add_system(player_shoot)
        .run();
}
fn start_up_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let bullet_pool = BulletPool::init(&mut commands, &asset_server, 500, true);
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("textures/ferris.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(UNIT_SIZE.0, UNIT_SIZE.1)),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(PLAYER_SPAWN_POINT.0, PLAYER_SPAWN_POINT.1, 0.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Player {
            bullets: bullet_pool,
            weapon_cool_down: Timer::from_seconds(PLAYER_WEAPON_COOL_DOWN, false),
        });
    EnemyPool::init(&mut commands, &asset_server, 100);
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
fn spawn_enemy(
    mut pool: ResMut<EnemyPool>,
    mut timer: ResMut<Timer>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Visibility), (With<AutoMove>, With<AutoShoot>)>,
) {
    if timer.finished() {
        let entity = pool.spawn();
        match entity {
            Some(entity) => {
                if let Ok((mut transform, mut visibility)) = query.get_mut(entity) {
                    let mut rng = thread_rng();
                    let x = rng.gen_range(ENEMY_X_RANGE);
                    transform.translation += Vec3::new(x, ENEMY_SPAWN_Y, 0.);
                    visibility.is_visible = true;
                }
            }
            None => warn!("runned out of the entities, how strong you are"),
        }
        timer.set(Box::new(Timer::from_seconds(3., false))).unwrap();
    } else {
        timer.tick(time.delta());
    }
}
fn player_shoot(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut player_query: Query<(&mut Player, &mut Transform), Without<AutoMove>>,
    mut bullet_query: Query<(&mut Visibility, &mut Transform), With<AutoMove>>,
) {
    for (mut player, player_transform) in player_query.iter_mut() {
        if !player.weapon_cool_down.finished() {
            player.weapon_cool_down.tick(time.delta());
            return;
        }
        if keys.any_pressed([KeyCode::Space]) {
            if let Some(bullet) = player.bullets.spawn() {
                if let Ok((mut visibility, mut transform)) = bullet_query.get_mut(bullet) {
                    visibility.is_visible = true;
                    let mut pos = player_transform.translation;
                    pos.y = pos.y + UNIT_SIZE.1 / 2. + PLAYER_BULLET_Y / 2.;
                    transform.translation = pos;
                }
            }
            player.weapon_cool_down = Timer::from_seconds(PLAYER_WEAPON_COOL_DOWN, false);
        }
    }
}
