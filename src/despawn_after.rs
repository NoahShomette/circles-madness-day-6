use bevy::prelude::*;

use crate::menu::GameState;

pub struct DespawnAfterPlugin;

impl Plugin for DespawnAfterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            despawn_after.run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct DespawnAfter {
    pub timer: Timer,
}

fn despawn_after(
    mut commands: Commands,
    time: Res<Time>,
    mut q_des: Query<(Entity, &mut DespawnAfter)>,
) {
    for (e, mut d) in q_des.iter_mut() {
        d.timer.tick(time.delta());
        if d.timer.finished() {
            commands.entity(e).despawn();
        }
    }
}
