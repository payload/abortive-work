use std::f32::consts::TAU;

use bevy::{
    input::{keyboard::KeyboardInput, system::exit_on_esc_system, ElementState},
    math::vec3,
    prelude::*,
};
pub use bevy_mod_picking::*;

use crate::entities::*;

pub struct UserInputPlugin;

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(PickingPlugin)
            .add_plugin(InteractablePickingPlugin)
            .add_plugin(HighlightablePickingPlugin)
            .add_system(exit_on_esc_system)
            .add_system(spawn_imp_on_key)
            .add_system(make_pickable)
            .add_system(click_boulder);
    }
}

fn spawn_imp_on_key(mut imp: ImpSpawn, mut keyboard_input_events: EventReader<KeyboardInput>) {
    for event in keyboard_input_events.iter() {
        if let Some(key_code) = event.key_code {
            if event.state == ElementState::Pressed && key_code == KeyCode::I {
                let a = TAU * fastrand::f32();
                let vec = vec3(a.cos(), 0.0, a.sin());
                imp.spawn(Imp::new(), Transform::from_xyz(vec.x, 0.0, vec.z));
            }
        }
    }
}

fn make_pickable(mut cmds: Commands, query: Query<Entity, Added<BoulderModel>>) {
    for entity in query.iter() {
        cmds.entity(entity)
            .insert_bundle((PickableMesh::default(), Interaction::None));
    }
}

fn click_boulder(
    models: Query<(&Parent, &Interaction), (Changed<Interaction>, With<BoulderModel>)>,
    mut boulders: Query<&mut Boulder>,
) {
    for (parent, interaction) in models.iter() {
        if let Interaction::Clicked = interaction {
            if let Ok(mut boulder) = boulders.get_mut(**parent) {
                boulder.marked_for_digging = !boulder.marked_for_digging;
            }
        }
    }
}
