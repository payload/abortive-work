use bevy::{ecs::system::SystemParam, prelude::*};

pub struct Destructable;

#[derive(SystemParam)]
pub struct Destructor<'w, 's> {
    cmds: Commands<'w, 's>,
    query: Query<'w, 's, (Entity, &'static Transform), With<Destructable>>,
}

impl<'w, 's> Destructor<'w, 's> {
    pub fn destruct_one_near(&mut self, pos: Vec3) {
        let near: Vec<_> = self
            .query
            .iter()
            .filter(|(_, transform)| pos.distance_squared(transform.translation) < 1.0)
            .collect();
        if near.len() > 0 {
            let index = fastrand::usize(0..near.len());
            if let Some((entity, _)) = near.get(index) {
                self.cmds.entity(*entity).despawn_recursive();
            }
        }
    }
}
