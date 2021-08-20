use bevy::{input::system::exit_on_esc_system, prelude::Plugin};

pub struct UserInputPlugin;

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(exit_on_esc_system);
    }
}