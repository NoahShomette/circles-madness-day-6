use bevy::audio::Volume;
use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_asset_loader::prelude::{AssetCollection, AssetCollectionApp};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::despawn_after::DespawnAfter;
use crate::menu::GameState;

use crate::{
    bullets::CommandsSpawnBullet,
    movement::{MoveSpeed, MoveTarget},
    player::Player,
    Cooldown, GameDef, Health, RemoveOnRespawn, TeamIdx, Weapon,
};

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.init_collection::<AiSoundAssets>();
        app.add_event::<AiDeathEvent>();
        app.add_systems(
            PostUpdate,
            handle_ai_sounds.run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct AiSoundAssets {
    #[asset(path = "sounds/explosion-2.ogg")]
    ai_killed: Handle<AudioSource>,
}

#[derive(Event, Debug, Default)]
pub struct AiDeathEvent {
    pub origin: Vec2,
}

#[derive(Component, Debug)]
pub struct Ai;

#[derive(Component, Debug)]
pub struct BigAi;

pub fn spawn_ais(
    time: Res<Time>,
    mut commands: Commands,
    mut timer: Local<Timer>,
    game_settings: Res<GameDef>,
) {
    timer.tick(time.delta());
    if timer.finished() {
        timer.set_duration(bevy::utils::Duration::from_secs_f32(
            game_settings.spawn_interval,
        ));
        timer.reset();
        let mut rng = rand::thread_rng();
        let chance = rand::thread_rng().gen_range(1..=100);
        if chance > 30 {
            commands.spawn((
                Transform {
                    translation: (Vec2::new(
                        rng.gen_range(-460_f32..460_f32),
                        rng.gen_range(-260_f32..260_f32),
                    ))
                    .extend(2f32),
                    ..default()
                },
                MoveSpeed(75f32),
                MoveTarget {
                    target: Some(Vec2::new(200f32, 200f32)),
                },
                Health {
                    current: 1f32,
                    max: 1f32,
                },
                Weapon {
                    bullets: 1u16,
                    max: 360_u16,
                    spread: 160_f32,
                },
                Cooldown {
                    start_time: 0.0,
                    duration: 2.0,
                },
                Ai,
                TeamIdx(1),
                RemoveOnRespawn,
            ));
        } else {
            commands.spawn((
                Transform {
                    translation: (Vec2::new(
                        rng.gen_range(-460_f32..460_f32),
                        rng.gen_range(-260_f32..260_f32),
                    ))
                    .extend(2f32),
                    ..default()
                },
                MoveSpeed(50f32),
                MoveTarget {
                    target: Some(Vec2::new(200f32, 200f32)),
                },
                Health {
                    current: 3f32,
                    max: 3f32,
                },
                Weapon {
                    bullets: 3u16,
                    max: 360_u16,
                    spread: 160_f32,
                },
                Cooldown {
                    start_time: 0.0,
                    duration: 2.0,
                },
                Ai,
                TeamIdx(1),
                RemoveOnRespawn,
                BigAi,
            ));
        }
    }
}

pub fn update_spawn_interval(
    time: Res<Time>,
    mut timer: Local<Timer>,
    mut game_settings: ResMut<GameDef>,
) {
    if timer.duration().as_millis() == 0 {
        *timer = Timer::new(bevy::utils::Duration::from_secs(4), TimerMode::Repeating);
    }
    timer.tick(time.delta());
    if timer.finished() {
        game_settings.spawn_interval *= game_settings.spawn_interval_multiplier_per_second;
        println!("spawn_interval {}", game_settings.spawn_interval);
    }
}

pub fn ai_move(
    time: Res<Time>,
    mut q_moves: Query<&mut MoveTarget, With<Ai>>,
    mut q_player: Query<&Transform, With<Player>>,
    mut timer: Local<Timer>,
) {
    if timer.duration().as_millis() == 0 {
        *timer = Timer::new(
            bevy::utils::Duration::from_millis(2360),
            TimerMode::Repeating,
        );
    }
    timer.tick(time.delta());
    if !timer.just_finished() {
        return;
    }
    let Some(player_position) = q_player.iter().next() else {
        return;
    };
    let mut rng = rand::thread_rng();
    for mut m in q_moves.iter_mut() {
        let t = rng.gen_range(0f32..1f32) * std::f32::consts::TAU;
        let offset = Vec2::new(t.cos(), t.sin()) * 200f32;
        m.target = Some(player_position.translation.xy() + offset);
    }
}

pub fn ai_fire(
    mut commands: Commands,
    time: Res<Time>,
    mut q_attackers: Query<
        (
            Entity,
            &Transform,
            &MoveTarget,
            &TeamIdx,
            &Cooldown,
            &Weapon,
        ),
        With<Ai>,
    >,
    mut q_player: Query<&Transform, With<Player>>,
    mut timer: Local<Timer>,
) {
    if timer.duration().as_millis() == 0 {
        *timer = Timer::new(bevy::utils::Duration::from_secs(1), TimerMode::Repeating);
    }
    timer.tick(time.delta());
    if !timer.just_finished() {
        return;
    }
    let Some(player_position) = q_player.iter().next() else {
        return;
    };
    let elapsed_seconds = time.elapsed_seconds();
    let mut rng: rand::rngs::ThreadRng = rand::thread_rng();
    let mut ais = q_attackers
        .iter_mut()
        .filter(|ai| ai.4.is_ready(elapsed_seconds))
        .collect::<Vec<_>>();
    ais.shuffle(&mut rng);

    let f32_number = elapsed_seconds / 35.0; // One extra one will shoot per every 35 seconds

    let amount_to_shoot: f32 = f32_number.clamp(1.0, 25.0);

    for (entity, transform, _, team, cooldown, weapon) in ais.iter().take(amount_to_shoot as usize)
    {
        let dot = rng.gen_range(0f32..1f32) * std::f32::consts::TAU;
        let offset = Vec2::new(dot.cos(), dot.sin()) * 50f32;

        let t_position = transform.translation.xy();
        if commands
            .spawn_bullet(
                *entity,
                t_position,
                ((player_position.translation.xy() + offset) - t_position).normalize_or_zero(),
                (*team).clone(),
                cooldown,
                &time,
                weapon.bullets,
                weapon.spread,
            )
            .is_ok()
        {
            commands.entity(*entity).insert(Cooldown {
                start_time: time.elapsed_seconds(),
                duration: cooldown.duration,
            });
        }
    }
}

pub fn handle_ai_sounds(
    bullet_assets: Res<AiSoundAssets>,
    mut commands: Commands,
    mut damaged_events: EventReader<AiDeathEvent>,
    listener: Query<&Transform, With<Player>>,
) {
    for e in damaged_events.iter() {
        let Ok(listener) = listener.get_single() else {
            return;
        };
        commands.spawn((
            SpatialAudioBundle {
                source: bullet_assets.ai_killed.clone(),
                settings: PlaybackSettings::ONCE.with_volume(Volume::new_relative(2.0)),
                spatial: SpatialSettings::new(
                    Transform::IDENTITY,
                    5f32,
                    ((e.origin - listener.translation.xy()).normalize_or_zero() * (5f32 / 2f32))
                        .extend(0f32),
                ),
            },
            DespawnAfter {
                timer: Timer::from_seconds(1_f32, TimerMode::Once),
            },
        ));
    }
}
