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

mod res_bundles;
pub use res_bundles::*;

mod mage;
pub use mage::*;

mod fireplace;
pub use fireplace::*;

mod pile;
pub use pile::*;

mod conveyor;
pub use conveyor::*;

pub mod tree;

pub struct NotGround;
pub struct Blocking;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(BoulderPlugin)
            .add_plugin(SmitheryPlugin)
            .add_plugin(ImpPlugin)
            .add_plugin(StoragePlugin)
            .add_plugin(MagePlugin)
            .add_plugin(FireplacePlugin)
            .add_plugin(PilePlugin)
            .add_plugin(ConveyorPlugin)
            .add_plugin(tree::Plugin)
            .add_plugin(GroundPlugin);
    }
}
