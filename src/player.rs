use bevy::{math::Vec3Swizzles, prelude::*, window::PrimaryWindow};
use bevy::audio::Volume;
use bevy_asset_loader::prelude::{AssetCollection, AssetCollectionApp};

use crate::despawn_after::DespawnAfter;
use crate::menu::GameState;
use crate::{
    bullets::CommandsSpawnBullet, menu::LastActivity, movement::MoveTarget, Cooldown, TeamIdx,
    Weapon,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_collection::<PlayerSoundAssets>();
        app.add_event::<PlayerDamagedEvent>();
        app.add_event::<PlayerPickupHealthEvent>();
        app.add_event::<PlayerPickupWeaponEvent>();
        app.add_systems(
            PostUpdate,
            handle_player_sounds.run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component, Debug)]
pub struct Player;

#[derive(AssetCollection, Resource)]
pub struct PlayerSoundAssets {
    #[asset(path = "sounds/player_damage.wav")]
    player_damaged: Handle<AudioSource>,
    #[asset(path = "sounds/player_heal.wav")]
    player_heal: Handle<AudioSource>,
    #[asset(path = "sounds/player_weapon_upgrade.wav")]
    player_weapon_upgrade: Handle<AudioSource>,
}

#[derive(Event, Debug, Default)]
pub struct PlayerDamagedEvent;

#[derive(Event, Debug, Default)]
pub struct PlayerPickupHealthEvent;

#[derive(Event, Debug, Default)]
pub struct PlayerPickupWeaponEvent;

pub fn handle_player_sounds(
    bullet_assets: Res<PlayerSoundAssets>,
    mut commands: Commands,
    mut damaged_events: EventReader<PlayerDamagedEvent>,
    mut health_events: EventReader<PlayerPickupHealthEvent>,
    mut weapon_events: EventReader<PlayerPickupWeaponEvent>,
    listener: Query<&Transform, With<Player>>,
) {
    for e in damaged_events.iter() {
        let Ok(listener) = listener.get_single() else{
            return;
        };
        commands.spawn((
            SpatialAudioBundle {
                source: bullet_assets.player_damaged.clone(),
                settings: PlaybackSettings::ONCE.with_volume(Volume::new_relative(5.0)),
                spatial: SpatialSettings::new(
                    Transform::IDENTITY,
                    5f32,
                    ((listener.translation.xy()).normalize_or_zero() * (5f32 / 2f32)).extend(0f32),
                ),
            },
            DespawnAfter {
                timer: Timer::from_seconds(1_f32, TimerMode::Once),
            },
        ));
    }

    for e in health_events.iter() {
        let Ok(listener) = listener.get_single() else{
            return;
        };
        commands.spawn((
            SpatialAudioBundle {
                source: bullet_assets.player_heal.clone(),
                settings: PlaybackSettings::ONCE.with_volume(Volume::new_relative(5.0)),
                spatial: SpatialSettings::new(
                    Transform::IDENTITY,
                    5f32,
                    ((listener.translation.xy()).normalize_or_zero() * (5f32 / 2f32)).extend(0f32),
                ),
            },
            DespawnAfter {
                timer: Timer::from_seconds(1_f32, TimerMode::Once),
            },
        ));
    }

    for e in weapon_events.iter() {
        let Ok(listener) = listener.get_single() else{
            return;
        };
        commands.spawn((
            SpatialAudioBundle {
                source: bullet_assets.player_weapon_upgrade.clone(),
                settings: PlaybackSettings::ONCE.with_volume(Volume::new_relative(10.0)),
                spatial: SpatialSettings::new(
                    Transform::IDENTITY,
                    5f32,
                    ((listener.translation.xy()).normalize_or_zero() * (5f32 / 2f32)).extend(0f32),
                ),
            },
            DespawnAfter {
                timer: Timer::from_seconds(1_f32, TimerMode::Once),
            },
        ));
    }
}

pub fn handle_mouse_to_move(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<Input<MouseButton>>,
    mut q_moves: Query<&mut MoveTarget, With<Player>>,
    camera: Query<(&GlobalTransform, &Camera)>,
) {
    if buttons.pressed(MouseButton::Left) {
        if let Some(position) = q_windows.single().cursor_position() {
            if let Some((camera_transform, camera)) = camera.iter().next() {
                let Some(position) = camera.viewport_to_world_2d(camera_transform, position) else {
                    return;
                };
                for mut m in q_moves.iter_mut() {
                    m.target = Some(position);
                }
            }
        }
    } else {
        for mut m in q_moves.iter_mut() {
            m.target = None;
        }
    }
}

pub fn handle_clicks_to_fire(
    mut commands: Commands,
    time: Res<Time>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<Input<MouseButton>>,
    mut q_attackers: Query<
        (
            Entity,
            &Transform,
            &MoveTarget,
            &TeamIdx,
            &Cooldown,
            &Weapon,
        ),
        With<Player>,
    >,
    camera: Query<(&GlobalTransform, &Camera)>,
    mut last_activity: ResMut<LastActivity>,
) {
    if buttons.pressed(MouseButton::Left) {
        if let Some(position) = q_windows.single().cursor_position() {
            if let Some((camera_transform, camera)) = camera.iter().next() {
                let Some(position) = camera.viewport_to_world_2d(camera_transform, position) else {
                    return;
                };
                for (entity, transform, _, team, cooldown, weapon) in q_attackers.iter_mut() {
                    let t_position = transform.translation.xy();
                    // TODO: rework bullet spawn to take place with an event
                    if commands
                        .spawn_bullet(
                            entity,
                            t_position,
                            (position - t_position).normalize_or_zero(),
                            team.clone(),
                            cooldown,
                            &time,
                            weapon.bullets,
                            weapon.spread,
                        )
                        .is_ok()
                    {
                        last_activity.0.reset();
                        commands.entity(entity).insert(Cooldown {
                            start_time: time.elapsed_seconds(),
                            duration: cooldown.duration,
                        });
                    }
                }
            }
        }
    }
}
