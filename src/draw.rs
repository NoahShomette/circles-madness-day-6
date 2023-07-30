use std::f32::consts::TAU;

use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_vector_shapes::prelude::*;

use crate::ai::BigAi;
use crate::{
    movement::{MoveDirection, MoveTarget},
    Cooldown, Health, Pickup, PickupKind, TeamIdx, Teams,
};

pub fn draw(
    teams: Res<Teams>,
    mut gizmos: Gizmos,
    q_movers: Query<(&Transform, &TeamIdx, Option<&BigAi>), With<MoveTarget>>,
) {
    for (transform, team, big_ai_option) in q_movers.iter() {
        if let Some(_) = big_ai_option {
            gizmos.circle_2d(transform.translation.xy(), 15f32, teams.colors[team.0].0);
        } else {
            gizmos.circle_2d(transform.translation.xy(), 5f32, teams.colors[team.0].0);
        }
    }
}

pub fn draw_bullets(
    teams: Res<Teams>,
    mut gizmos: Gizmos,
    q_movers: Query<(&Transform, &TeamIdx), With<MoveDirection>>,
) {
    for (transform, team) in q_movers.iter() {
        gizmos.circle_2d(transform.translation.xy(), 2f32, teams.colors[team.0].1);
    }
}

pub fn draw_health(
    mut painter: ShapePainter,
    q_movers: Query<(&Transform, &Health, &TeamIdx, Option<&BigAi>)>,
) {
    for (transform, health, team, big_ai_option) in q_movers.iter() {
        if health.max <= health.current {
            continue;
        }
        painter.set_translation(transform.translation);

        let start_angle = 0f32 * 3.0;
        let end_angle = start_angle + ((health.current / health.max) * TAU);

        painter.thickness = 1f32;
        painter.hollow = true;
        painter.color = Color::CRIMSON * 3f32;
        painter.cap = Cap::None;
        let mut radius = 10f32;

        if let Some(_) = big_ai_option {
            radius = 20f32;
        }
        painter.arc(radius, start_angle, end_angle);
    }
}
pub fn draw_cooldown(
    time: Res<Time>,
    mut painter: ShapePainter,
    q_movers: Query<(&Transform, &Cooldown, &TeamIdx, Option<&BigAi>)>,
) {
    for (transform, cooldown, team, big_ai_option) in q_movers.iter() {
        if cooldown.start_time + cooldown.duration < time.elapsed_seconds() {
            continue;
        }
        let ratio = (time.elapsed_seconds() - cooldown.start_time) / cooldown.duration;
        painter.set_translation(transform.translation);

        let start_angle = 0f32 * 3.0;
        let end_angle = start_angle + (ratio * TAU);
        let mut radius = 13f32;
        if let Some(_) = big_ai_option {
            radius = 25f32;
        }
        painter.thickness = 1f32;
        painter.hollow = true;
        painter.color = Color::WHITE;
        painter.cap = Cap::None;
        painter.arc(radius, start_angle, end_angle);
    }
}

pub fn draw_pickups(time: Res<Time>, mut gizmos: Gizmos, q_movers: Query<(&Transform, &Pickup)>) {
    for (transform, pickup) in q_movers.iter() {
        match pickup.0 {
            PickupKind::Health(_) => {
                gizmos.circle_2d(
                    transform.translation.xy(),
                    2f32 + (time.elapsed_seconds() * 3f32).sin(),
                    Color::BLUE * 3f32,
                );
            }
            PickupKind::Weapon(_) => {
                gizmos.circle_2d(
                    transform.translation.xy(),
                    2f32 + (time.elapsed_seconds() * 3f32).sin(),
                    Color::RED * 3f32,
                );
            }
        }
    }
}
