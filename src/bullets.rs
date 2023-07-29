use bevy::{ecs::system::Command, math::Vec3Swizzles, prelude::*};
use bevy_asset_loader::prelude::*;

use crate::{
    despawn_after::DespawnAfter,
    menu::GameState,
    movement::{MoveDirection, MoveSpeed},
    player::Player,
    Cooldown, RemoveOnRespawn, TeamIdx,
};

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.init_collection::<BulletAssets>();
        app.add_systems(
            PostUpdate,
            bullet_sounds.run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(AssetCollection, Resource)]
struct BulletAssets {
    #[asset(path = "sounds/zap.ogg")]
    pew1: Handle<AudioSource>,
}

#[derive(Event, Debug)]
pub struct EventBulletSpawn {
    pub origin: Vec2,
}

pub struct SpawnBulletCommand {
    from_entity: Entity,
    team: TeamIdx,
    from_position: Vec2,
    to_direction: Vec2,
    number_of_bullets: u16,
    bullet_spread: f32,
}

#[derive(Component)]
pub struct BulletOwner {
    pub entity: Entity,
}

impl Command for SpawnBulletCommand {
    fn apply(self, world: &mut World) {
        for n in 0..self.number_of_bullets as i16 {
            let rotation =
                Vec2::from_angle(std::f32::consts::TAU / self.number_of_bullets as f32 * n as f32);
            world.spawn((
                Transform {
                    translation: self.from_position.extend(2f32),
                    ..default()
                },
                MoveSpeed(400f32),
                MoveDirection(rotation.rotate(self.to_direction)),
                DespawnAfter {
                    timer: Timer::from_seconds(2_f32, TimerMode::Once),
                },
                BulletOwner {
                    entity: self.from_entity,
                },
                self.team,
                RemoveOnRespawn,
            ));
        }
        world.send_event(EventBulletSpawn {
            origin: self.from_position,
        });
    }
}

pub trait CommandsSpawnBullet {
    fn spawn_bullet(
        &mut self,
        from_entity: Entity,
        from_position: Vec2,
        direction: Vec2,
        team: TeamIdx,
        cooldown: &Cooldown,
        time: &Res<Time>,
        number_of_bullets: u16,
        bullet_spread: f32,
    ) -> Result<&mut Self, ()>;
}

impl CommandsSpawnBullet for Commands<'_, '_> {
    fn spawn_bullet(
        &mut self,
        from_entity: Entity,
        from_position: Vec2,
        direction: Vec2,
        team: TeamIdx,
        cooldown: &Cooldown,
        time: &Res<Time>,
        number_of_bullets: u16,
        bullet_spread: f32,
    ) -> Result<&mut Self, ()> {
        if direction == Vec2::ZERO {
            return Err(());
        }
        if time.elapsed_seconds() < cooldown.start_time + cooldown.duration {
            return Err(());
        }

        self.add(SpawnBulletCommand {
            from_entity,
            from_position,
            to_direction: direction,
            team,
            number_of_bullets,
            bullet_spread,
        });
        Ok(self)
    }
}

fn bullet_sounds(
    time: Res<Time>,
    bullet_assets: Res<BulletAssets>,
    mut commands: Commands,
    mut ev_bullets: EventReader<EventBulletSpawn>,
    listener: Query<&Transform, With<Player>>,
) {
    for e in ev_bullets.iter() {
        let listener = listener.single();
        commands.spawn((
            SpatialAudioBundle {
                source: bullet_assets.pew1.clone(),
                settings: PlaybackSettings::ONCE,
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
