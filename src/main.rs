use bevy::prelude::*;

mod entities;
use entities::*;

mod systems;
use systems::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AnimationsPlugin)
        .add_plugin(EntitiesPlugin)
        .add_plugin(UserInputPlugin)
        .add_startup_system(spawn_level_1)
        .add_startup_system(spawn_camera)
        .run();

    // a portal which produces imps next to it
    // keyboard keys can trigger production at portal
    //
    // imps, who can dig, need sleep, haul things to better places
    // an imp can dig faster with a tool
    // tools can break
    //
    // mouse cursor can place things, delete things and change things
    // those things are:
    //  boulders of stone, coal, iron, gold can be marked for digging
    //  sleeping place
    //  treasure place (goal drop place, high prio)
    //  smithery (with a coal drop place and a iron drop place, high prio) produces tools
    //  storage place (drop place for anything, low prio)
    //
    // counter for stuff in treasure, storage
}

fn spawn_level_1(mut boulder: BoulderSpawn, mut smithery: SmitherySpawn, mut imp: ImpSpawn) {
    use BoulderMaterial::*;

    boulder.spawn(Boulder { material: Stone }, at(3, 3));
    boulder.spawn(Boulder { material: Coal }, at(2, 3));
    boulder.spawn(Boulder { material: Stone }, at(2, 2));
    boulder.spawn(Boulder { material: Gold }, at(1, 3));
    boulder.spawn(Boulder { material: Iron }, at(1, 2));

    smithery.spawn(Smithery, at(-3, 2));

    imp.spawn(Imp, at(0, 0));

    fn at(x: i32, z: i32) -> Transform {
        Transform::from_xyz(x as f32, 0.0, z as f32)
    }
}

fn spawn_camera(mut cmds: Commands) {
    cmds.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, 10.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
