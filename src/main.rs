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
const BULLET_Y: f32 = 37.;
const BULLET_X: f32 = 13.;
const PLAYER_WEAPON_COOL_DOWN: f32 = 0.3;
const ENEMY_MOVE_SPEED: f32 = 0.4;
const ENEMY_SHOOT_COOL_DOWN: f32 = 1.5;
const PLAYER_BULLET_SPEED: f32 = 2.;
const ENEMY_BULLET_SPEED: f32 = 2.;
use rand::prelude::*;
/// Camp come from auto translation
/// translate from "阵营" in chinese
#[derive(Component, Debug)]
enum Camp {
    Player,
    Enemy,
    Neutral,
}
/// wants to using UnitType at first time
/// but it contains one field called Unit inside
/// it is really weird to have a unit inside unit type
/// so I changed it to Entity Type
#[derive(Component, Debug)]
enum EntityType {
    Item,
    Unit,
    Bullet,
}
#[derive(Component, Debug)]
struct Movable {
    direction: Vec3,
    speed: f32,
}
/// when the automatic is true
/// the unit will shoot once per cooldown
#[derive(Component)]
struct Shooter {
    automatic: bool,
    weapon_cool_down: Timer,
}
/// really run out of the words to describe this bundle
/// but you needed to know that everything that is visible in our game
/// should all have this bundle
#[derive(Bundle)]
struct CampAndTypeBundle {
    camp: Camp,
    entity_type: EntityType,
}
#[derive(Bundle)]
struct Enemy {
    shooter: Shooter,
    movable: Movable,
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
                    movable: Movable {
                        direction: Vec3::new(0., 0., 0.),
                        speed: ENEMY_MOVE_SPEED,
                    },
                    shooter: Shooter {
                        automatic: true,
                        weapon_cool_down: Timer::from_seconds(ENEMY_SHOOT_COOL_DOWN, false),
                    },
                })
                .insert_bundle(CampAndTypeBundle {
                    camp: Camp::Enemy,
                    entity_type: EntityType::Unit,
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
    pub fn init(mut commands: Commands, size: usize) {
        let mut pool = VecDeque::new();
        for _ in 0..size {
            let entity = commands
                .spawn_bundle(SpriteBundle {
                    visibility: Visibility { is_visible: false },
                    ..Default::default()
                })
                .insert(Movable {
                    direction: Vec3::new(0., 0., 0.),
                    speed: PLAYER_BULLET_SPEED,
                })
                .insert_bundle(CampAndTypeBundle {
                    camp: Camp::Neutral,
                    entity_type: EntityType::Bullet,
                })
                .id();
            pool.push_back(entity);
        }
        commands.insert_resource(Self(pool))
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
        .add_system(moving)
        .add_system(shooting)
        .add_system(recycle_bullet_when_it_is_out_of_boundary)
        .run();
}
fn start_up_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
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
        .insert(Shooter {
            weapon_cool_down: Timer::from_seconds(PLAYER_WEAPON_COOL_DOWN, false),
            automatic: false,
        })
        .insert(Movable {
            direction: Vec3::new(0., 0., 0.),
            speed: PLAYER_MOVE_SPEED,
        })
        .insert_bundle(CampAndTypeBundle {
            camp: Camp::Player,
            entity_type: EntityType::Unit,
        });
    EnemyPool::init(&mut commands, &asset_server, 100);
    BulletPool::init(commands, 5000);
}
fn move_player(keys: Res<Input<KeyCode>>, mut chars: Query<(&Camp, &mut Movable, &EntityType)>) {
    for mut char in chars.iter_mut() {
        match char.0 {
            Camp::Player => match char.2 {
                EntityType::Unit => {
                    let mut direction = Vec3::new(0., 0., 0.);
                    if keys.pressed(KeyCode::W) {
                        direction.y += 1.;
                    }
                    if keys.pressed(KeyCode::S) {
                        direction.y -= 1.;
                    }
                    if keys.pressed(KeyCode::A) {
                        direction.x -= 1.;
                    }
                    if keys.pressed(KeyCode::D) {
                        direction.x += 1.;
                    }
                    char.1.direction = direction;
                }
                _ => continue,
            },
            _ => {}
        }
    }
}
fn spawn_enemy(
    mut pool: ResMut<EnemyPool>,
    mut timer: ResMut<Timer>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Visibility, &mut Movable)>,
) {
    if timer.finished() {
        let entity = pool.spawn();
        match entity {
            Some(entity) => {
                if let Ok((mut transform, mut visibility, mut movable)) = query.get_mut(entity) {
                    let mut rng = thread_rng();
                    let x = rng.gen_range(ENEMY_X_RANGE);
                    transform.translation += Vec3::new(x, ENEMY_SPAWN_Y, 0.);
                    visibility.is_visible = true;
                    movable.direction.y -= 1.;
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
    mut bullets: ResMut<BulletPool>,
    asset_server: Res<AssetServer>,
    mut player_query: Query<(&mut Shooter, &mut Transform, &Camp)>,
    mut bullet_query: Query<
        (
            &mut Movable,
            &mut Transform,
            &mut Visibility,
            &mut Camp,
            &mut Handle<Image>,
        ),
        Without<Shooter>,
    >,
) {
    for (mut player, player_transform, camp) in player_query.iter_mut() {
        match camp {
            Camp::Player => {
                if !player.weapon_cool_down.finished() {
                    player.weapon_cool_down.tick(time.delta());
                    return;
                }
                if keys.any_pressed([KeyCode::Space]) {
                    if let Some(bullet) = bullets.spawn() {
                        if let Ok((
                            mut movable,
                            mut transform,
                            mut visibility,
                            mut camp,
                            mut texture,
                        )) = bullet_query.get_mut(bullet)
                        {
                            visibility.is_visible = true;
                            let mut pos = player_transform.translation;
                            pos.y = pos.y + UNIT_SIZE.1 / 2. + BULLET_Y / 2.;
                            transform.translation = pos;
                            movable.direction = Vec3::new(0., 1., 0.);
                            movable.speed = PLAYER_BULLET_SPEED;
                            *texture.as_mut() = asset_server.load("textures/bullet.png");
                            *camp.as_mut() = Camp::Player;
                        }
                    }
                    player.weapon_cool_down = Timer::from_seconds(PLAYER_WEAPON_COOL_DOWN, false);
                }
            }
            _ => continue,
        }
    }
}

fn moving(mut query: Query<(&mut Transform, &Movable)>) {
    for (mut transform, movable) in query.iter_mut() {
        transform.translation += movable.direction * movable.speed;
    }
}

fn shooting(
    mut query: Query<(&Transform, &mut Shooter, &Camp, &Visibility)>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut bullets: ResMut<BulletPool>,
    mut bullet_query: Query<
        (
            &mut Movable,
            &mut Transform,
            &mut Visibility,
            &mut Camp,
            &mut Handle<Image>,
        ),
        Without<Shooter>,
    >,
) {
    for (shooter_transform, mut shooter, shooter_camp, visibility) in query.iter_mut() {
        match *shooter_camp {
            Camp::Enemy => {
                if !shooter.weapon_cool_down.finished() {
                    shooter.weapon_cool_down.tick(time.delta());
                    continue;
                }
                if !visibility.is_visible {
                    continue;
                }
                if let Some(bullet) = bullets.spawn() {
                    if let Ok((mut movable, mut transform, mut visibility, mut camp, mut texture)) =
                        bullet_query.get_mut(bullet)
                    {
                        visibility.is_visible = true;
                        transform.translation = transform.translation
                            + Vec3::new(
                                shooter_transform.translation.x,
                                shooter_transform.translation.y - ENEMY_SIZE.1 / 2. - BULLET_Y / 2.,
                                0.,
                            );
                        movable.direction = Vec3::new(0., -1., 0.);
                        movable.speed = ENEMY_BULLET_SPEED;
                        *texture.as_mut() = asset_server.load("textures/enemy_bullet.png");
                        *camp.as_mut() = Camp::Enemy;
                    }
                }
                shooter.weapon_cool_down = Timer::from_seconds(PLAYER_WEAPON_COOL_DOWN, false);
            }
            _ => continue,
        }
    }
}

fn recycle_bullet_when_it_is_out_of_boundary(
    mut query: Query<(Entity, &Transform, &mut Movable, &mut Camp, &EntityType)>,
    mut bullets: ResMut<BulletPool>,
) {
    for (id, transform, mut movable, mut camp, entity_type) in query.iter_mut() {
        match entity_type {
            EntityType::Bullet => {
                let top = (WINDOW_SIZE.1 + BULLET_Y) / 2.;
                let bottom = -(WINDOW_SIZE.1 + BULLET_Y) / 2.;
                let left = -(WINDOW_SIZE.0 + BULLET_X) / 2.;
                let right = (WINDOW_SIZE.0 + BULLET_X) / 2.;
                let (x, y) = (transform.translation.x, transform.translation.y);
                if y >= top || y <= bottom || x <= left || x >= right {
                    movable.direction = Vec3::new(0., 0., 0.);
                    *camp.as_mut() = Camp::Neutral;
                    bullets.recycle(id);
                }
            }
            _ => continue,
        }
    }
}
