use bevy::prelude::Plugin;

mod boulder;
pub use boulder::*;

mod smithery;
pub use smithery::*;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(BoulderPlugin).add_plugin(SmitheryPlugin);
    }
}
