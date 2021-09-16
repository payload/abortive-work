mod animations;
pub use animations::*;

mod user_input;
use bevy::prelude::Plugin;
pub use user_input::*;

mod building_tool;
pub use building_tool::*;

mod store;
pub use store::*;

mod camera;
pub use camera::*;

mod meshes;
pub use meshes::*;

mod augmentation;
pub use augmentation::*;

mod map_gen;
pub use map_gen::*;

mod destructor;
pub use destructor::*;

mod focus;
pub use focus::*;

mod brain;
pub use brain::*;

mod interact_with_focus;

pub struct SystemsPlugin;
impl Plugin for SystemsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(interact_with_focus::ModPlugin);
    }
}