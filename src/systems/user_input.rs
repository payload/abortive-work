use std::f32::consts::TAU;

use bevy::{
    input::{keyboard::KeyboardInput, system::exit_on_esc_system, ElementState},
    math::vec3,
    prelude::*,
};

use crate::entities::*;

pub struct UserInputPlugin;

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(exit_on_esc_system)
            .add_system(spawn_imp_on_key);
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
