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
        .add_plugin(CameraPlugin)
        .add_plugin(AugmentationPlugin)
        .add_startup_system(spawn_level_1)
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

fn spawn_level_1(
    mut ground: GroundSpawn,
    mut boulder: BoulderSpawn,
    mut smithery: SmitherySpawn,
    mut imp: ImpSpawn,
    mut storage: StorageSpawn,
    mut mage: MageSpawn,
    mut camera: CameraSpawn,
    mut fireplace: FireplaceSpawn,
) {
    use BoulderMaterial::*;

    ground.spawn(Ground, at(0, 0));

    boulder.spawn(Boulder::new(Stone), at(6, 3));
    boulder.spawn(Boulder::new(Stone), at(6, 2));
    boulder.spawn(Boulder::new(Stone), at(6, 1));

    boulder.spawn(Boulder::new(Iron), at(-6, 2));
    boulder.spawn(Boulder::new(Iron), at(-6, 1));
    boulder.spawn(Boulder::new(Iron), at(-6, 0));

    boulder.spawn(Boulder::new(Coal), at(1, 4));

    smithery.spawn(Smithery::new(), at(-3, -2));

    imp.spawn(Imp::new(), at(0, 0));

    mage.spawn(Mage::new(), at(-1, 0))
        .insert(CameraTracking::new(0.0, 10.0, -3.0));
    camera.spawn();

    storage.spawn(Storage::new(), at(0, -1));

    fireplace.spawn(Fireplace::new(), at(0, 0));

    fn at(x: i32, z: i32) -> Transform {
        Transform::from_xyz(x as f32, 0.0, z as f32)
    }
}
