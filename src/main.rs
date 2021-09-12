use bevy::prelude::*;

mod entities;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use entities::*;

mod systems;
use noice::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm,
};
use systems::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AnimationsPlugin)
        .add_plugin(EntitiesPlugin)
        .add_plugin(UserInputPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(AugmentationPlugin)
        .add_plugin(FocusPlugin)
        .add_plugin(DebugLinesPlugin)
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
    mut mage: MageSpawn,
    mut camera: CameraSpawn,
    mut fireplace: FireplaceSpawn,
    mut pile: PileSpawn,
    mut conveyor: ConveyorSpawn,
) {
    use BoulderMaterial::*;

    let noise = Fbm::new();
    let map = PlaneMapBuilder::new(&noise)
        .set_size(128, 128)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();

    let (w, h) = map.size();
    let hx = -0.5 * w as f32;
    let hz = -0.5 * h as f32;

    for y in 0..h {
        for x in 0..w {
            let v = map.get_value(x, y);
            let x = x as f32;
            let z = y as f32;
            let transform = Transform::from_xyz(x + hx, 0.0, z + hz);

            if v > 0.3 && 0.4 > v {
                boulder.spawn(Boulder::new(Stone), transform);
            } else if v > 0.4 && 0.45 > v {
                boulder.spawn(Boulder::new(Coal), transform);
            } else if v > 0.45 && 0.47 > v {
                boulder.spawn(Boulder::new(Stone), transform);
            } else if v > 0.47 && 0.49 > v {
                boulder.spawn(Boulder::new(Iron), transform);
            } else if v > 0.48 && 0.53 > v && fastrand::f32() < 0.05 {
                boulder.spawn(Boulder::new(Gold), transform);
            } else if v > 0.47 && 1.0 > v {
                boulder.spawn(Boulder::new(Stone), transform);
            }
        }
    }

    let center = Vec3::new(7.0, 0.0, 6.0);
    let at = |x: i32, z: i32| -> Transform {
        Transform::from_xyz(center.x + x as f32, 0.0, center.z + z as f32)
    };

    ground.spawn(Ground, at(0, 0));

    smithery.spawn(Smithery::new(), at(3, -2));

    imp.spawn(Imp::new(), at(0, 0));

    mage.spawn(Mage::new(), at(-1, 0))
        .insert(CameraTracking::new(0.0, 10.0, -3.0));
    camera.spawn(center);

    fireplace.spawn(Fireplace::new(), at(0, 0));

    pile.spawn(Pile::new(Thing::Coal, 1.0), at(0, 1));

    conveyor.spawn_line(at(1, -1).translation, at(-1, -3).translation);
}
