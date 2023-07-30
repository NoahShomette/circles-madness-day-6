#![allow(clippy::too_many_arguments, clippy::type_complexity)]

pub mod ai;
mod bullets;
pub mod despawn_after;
pub mod draw;
pub mod menu;
pub mod movement;
pub mod player;
pub mod utils;

use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    math::vec2,
    prelude::*,
    render::camera::ScalingMode,
};
use bevy_asset_loader::prelude::AssetCollectionApp;
use bevy_vector_shapes::prelude::*;
use rand::prelude::*;

use ai::*;
use bullets::*;
use despawn_after::*;
use draw::*;
use menu::*;
use movement::*;
use player::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(DespawnAfterPlugin)
        .add_plugins(Game)
        .run();
}
#[derive(Component, Debug)]
pub struct RemoveOnRespawn;

#[derive(Component, Debug)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Debug)]
pub struct Weapon {
    pub bullets: u16,
    pub max: u16,
    pub spread: f32,
}

#[derive(Component, Debug)]
pub struct Cooldown {
    pub start_time: f32,
    pub duration: f32,
}
impl Cooldown {
    fn is_ready(&self, elapsed_seconds: f32) -> bool {
        self.start_time + self.duration < elapsed_seconds
    }
}

pub struct Game;

#[derive(Component, Clone, Copy)]
pub struct TeamIdx(pub usize);

#[derive(Resource)]
pub struct Teams {
    pub colors: Vec<(Color, Color)>,
}

impl Default for Teams {
    fn default() -> Self {
        Self {
            colors: vec![
                (Color::WHITE * 5f32, Color::GREEN * 5f32),
                (Color::ORANGE * 5f32, Color::RED * 5f32),
            ],
        }
    }
}

#[derive(Clone)]
enum PickupKind {
    Health(f32),
    Weapon(u16),
}

#[derive(Component, Clone)]
pub struct Pickup(PickupKind);

#[derive(Event)]
pub struct EventTryApplyDamages(pub Entity, pub f32);

#[derive(Resource)]
pub struct GameDef {
    pub spawn_interval: f32,
    pub initial_spawn_interval: f32,
    pub spawn_interval_multiplier_per_second: f32,
}

impl Default for GameDef {
    fn default() -> Self {
        Self {
            spawn_interval: 5f32,
            initial_spawn_interval: 5f32, // Messy messy
            spawn_interval_multiplier_per_second: 0.9f32,
        }
    }
}

impl Plugin for Game {
    fn build(&self, app: &mut App) {
        app.add_plugins(Shape2dPlugin::default());
        app.add_plugins(BulletPlugin)
            .add_plugins(MenuPlugin)
            .add_plugins(PlayerPlugin)
            .add_plugins(AiPlugin);
        app.init_resource::<GameDef>().init_resource::<Teams>();
        app.add_event::<EventBulletSpawn>()
            .add_event::<EventTryApplyDamages>();
        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            (
                (player_respawn),
                (
                    /*handle_mouse_to_move, */ handle_clicks_to_fire,
                    wasd_movement,
                ),
                (
                    move_targets,
                    move_direction,
                    spawn_ais,
                    ai_fire,
                    ai_move,
                    update_spawn_interval,
                ),
                (try_apply_damages,),
                (
                    collisions_player_pickups,
                    collisions_bullet_health,
                    draw,
                    draw_bullets,
                    draw_health,
                    draw_cooldown,
                    draw_pickups,
                ),
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn player_respawn(
    mut commands: Commands,
    mut q: ParamSet<(
        Query<Entity, With<Player>>,
        Query<Entity, With<RemoveOnRespawn>>,
    )>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if q.p0().iter().next().is_some() {
        return;
    }
    // Remove extra stuff
    for e in q.p1().iter() {
        commands.entity(e).despawn();
    }
    // Spawn player
    commands.spawn((
        Transform {
            translation: Vec2::ZERO.extend(2f32),
            ..default()
        },
        MoveSpeed(130f32),
        MoveDirection(Vec2::ZERO),
        MoveTarget {
            target: Some(Vec2::new(0f32, 0f32)),
        },
        Health {
            current: 1.5f32,
            max: 1.5f32,
        },
        Weapon {
            bullets: 1u16,
            max: 36u16,
            spread: 15_f32,
        },
        Cooldown {
            start_time: 0.0,
            duration: 0.5,
        },
        Player,
        TeamIdx(0),
    ));
    // Go back to menu
    // This system is called at the begining of the game and triggers the menu,
    // The game should be started in the Playing state to avoid having a double menu
    // until this is somehow fixed
    game_state.0 = Some(GameState::Menu);
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2dBundle {
            projection: OrthographicProjection {
                scaling_mode: ScalingMode::AutoMin {
                    min_width: 512.0,
                    min_height: 512.0,
                },
                ..default()
            },
            camera: Camera {
                hdr: true, // 1. HDR is required for bloom
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
            ..default()
        },
        BloomSettings::default(), // 3. Enable bloom for the camera
    ));

    commands.spawn(SpriteBundle {
        texture: asset_server.load("bg.jpg"),
        transform: Transform::from_xyz(0.0, 20.0, 0.0),
        sprite: Sprite {
            custom_size: Some(vec2(2048.0 * 1.5 / 2.0, 2048.0 / 2.0)),
            ..default()
        },
        ..default()
    });
}

pub fn collisions_bullet_health(
    mut commands: Commands,
    mut events_try_damage: EventWriter<EventTryApplyDamages>,
    q_bullets: Query<(Entity, &Transform, &BulletOwner)>,
    q_health: Query<(Entity, &Transform, &Health)>,
) {
    for (e_bullet, bullet_position, bullet_owner) in q_bullets.iter() {
        for (e, t, _) in q_health.iter() {
            if bullet_owner.entity != e
                && bullet_position.translation.distance(t.translation) < 20f32
            {
                commands.entity(e_bullet).despawn();
                events_try_damage.send(EventTryApplyDamages(e, 0.25f32));
                continue;
            }
        }
    }
}
pub fn collisions_player_pickups(
    mut commands: Commands,
    q_pickups: Query<(Entity, &Transform, &Pickup)>,
    mut q_stats: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            &mut Weapon,
            Option<&Player>,
        ),
        Without<Pickup>,
    >,
    mut health_events: EventWriter<PlayerPickupHealthEvent>,
    mut weapon_events: EventWriter<PlayerPickupWeaponEvent>,
) {
    for (e, t, mut health, mut weapon, option_player) in q_stats.iter_mut() {
        for (e_pickup, bullet_position, pickup) in q_pickups.iter() {
            if bullet_position.translation.distance(t.translation) < 20f32 {
                let is_player_picking_up = option_player.is_some();
                match pickup.0 {
                    PickupKind::Health(health_value) => {
                        health.current += health_value;
                        health.current = health.current.min(health.max);
                        commands.entity(e_pickup).despawn();
                        if is_player_picking_up {
                            health_events.send_default();
                        }
                        continue;
                    }
                    PickupKind::Weapon(bullet_increase) => {
                        weapon.bullets += bullet_increase;
                        weapon.bullets = weapon.bullets.min(weapon.max);
                        commands.entity(e_pickup).despawn();
                        if is_player_picking_up {
                            weapon_events.send_default();
                        }
                        continue;
                    }
                }
            }
        }
    }
}

pub fn try_apply_damages(
    mut commands: Commands,
    mut events_try_damage: EventReader<EventTryApplyDamages>,
    mut q_health: Query<(
        Entity,
        &Transform,
        &mut Health,
        Option<&BigAi>,
        Option<&Player>,
    )>,
    mut player_damage_events: EventWriter<PlayerDamagedEvent>,
    mut ai_killed: EventWriter<AiDeathEvent>,
) {
    let mut deleted_entities = Vec::new();
    for ev in events_try_damage.iter() {
        if deleted_entities.contains(&ev.0) {
            continue;
        }
        match q_health.get_mut(ev.0) {
            Ok((e, transform, mut health, option_big_ai, option_player)) => {
                if let Some(_) = option_player {
                    player_damage_events.send_default();
                }
                health.current -= 0.25f32;
                // TODO: fire event touched to spawn particles!
                if dbg!(health.current) <= 0f32 {
                    if let Some(_) = option_player {
                    } else {
                        ai_killed.send_default();
                    }
                    commands.entity(e).despawn();
                    deleted_entities.push(ev.0);
                    let chance = thread_rng().gen_range(1..=100);
                    let amount_amount_to_spawn = match option_big_ai {
                        None => 1,
                        Some(_) => 3,
                    };
                    if chance > 60 {
                        for i in 0..amount_amount_to_spawn {
                            commands.spawn((
                                Pickup(PickupKind::Health(0.25f32)),
                                Transform::from_translation(Vec3::new(
                                    transform.translation.x + (i * 2) as f32,
                                    transform.translation.y + (i * 2) as f32,
                                    transform.translation.z,
                                )),
                                RemoveOnRespawn,
                            ));
                        }
                    } else {
                        for i in 0..amount_amount_to_spawn {
                            commands.spawn((
                                Pickup(PickupKind::Weapon(1)),
                                Transform::from_translation(Vec3::new(
                                    transform.translation.x + (i * 2) as f32,
                                    transform.translation.y + (i * 2) as f32,
                                    transform.translation.z,
                                )),
                                RemoveOnRespawn,
                            ));
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
