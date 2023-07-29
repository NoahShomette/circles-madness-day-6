use bevy::{math::Vec3Swizzles, prelude::*, render::render_resource::CommandEncoderDescriptor};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::{
    bullets::CommandsSpawnBullet,
    movement::{MoveSpeed, MoveTarget},
    player::Player,
    Cooldown, GameDef, Health, RemoveOnRespawn, TeamIdx, Weapon,
};

#[derive(Component, Debug)]
pub struct Ai;

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
        commands.spawn((
            Transform {
                translation: (Vec2::new(
                    rand::thread_rng().gen_range(-460_f32..460_f32),
                    rand::thread_rng().gen_range(-260_f32..260_f32),
                ))
                .extend(2f32),
                ..default()
            },
            MoveSpeed(100f32),
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
    for (entity, transform, _, team, cooldown, weapon) in ais.iter().take(1) {
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
