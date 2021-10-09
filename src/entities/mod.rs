use bevy::prelude::Plugin;

mod boulder;
pub use boulder::*;

mod imp;
pub use imp::*;

mod ground;
pub use ground::*;

mod res_bundles;
pub use res_bundles::*;

mod mage;
pub use mage::*;

mod pile;
pub use pile::*;

mod conveyor;
pub use conveyor::*;

pub mod dump;
pub mod generator;
pub mod ritual_site;
pub mod sign;
pub mod transformer;
pub mod tree;

pub struct NotGround;
pub struct Blocking;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(BoulderPlugin)
            .add_plugin(ImpPlugin)
            .add_plugin(MagePlugin)
            .add_plugin(PilePlugin)
            .add_plugin(ConveyorPlugin)
            .add_plugin(tree::Plugin)
            .add_plugin(ritual_site::Plugin)
            .add_plugin(GroundPlugin)
            .add_plugin(dump::Plugin)
            .add_plugin(generator::Plugin)
            .add_plugin(transformer::Plugin)
            .add_plugin(sign::ModPlugin);
    }
}
