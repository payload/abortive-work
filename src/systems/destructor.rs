use bevy::{ecs::system::SystemParam, prelude::*};

pub struct Destructable;

#[derive(SystemParam)]
pub struct Destructor<'w, 's> {
    cmds: Commands<'w, 's>,
}

impl<'w, 's> Destructor<'w, 's> {
    pub fn destruct_some(&mut self, entity: Option<Entity>) {
        if let Some(entity) = entity {
            self.cmds.entity(entity).despawn_recursive();
        }
    }
}
