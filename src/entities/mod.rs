use bevy::prelude::Plugin;

mod boulder;
pub use boulder::*;

mod smithery;
pub use smithery::*;

mod imp;
pub use imp::*;

mod storage;
pub use storage::*;

mod ground;
pub use ground::*;

pub struct NotGround;
pub struct Blocking;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(BoulderPlugin)
            .add_plugin(SmitheryPlugin)
            .add_plugin(ImpPlugin)
            .add_plugin(StoragePlugin)
            .add_plugin(GroundPlugin);
    }
}
