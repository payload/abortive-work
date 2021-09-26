use std::f32::consts::TAU;

use bevy::prelude::*;

mod entities;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use entities::tree::Tree;
use entities::*;

mod systems;
use noice::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm,
};
use systems::*;

use crate::entities::{dump::Dump, ritual_site::RitualSite};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AnimationsPlugin)
        .add_plugin(EntitiesPlugin)
        .add_plugin(UserInputPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(AugmentationPlugin)
        .add_plugin(FocusPlugin)
        .add_plugin(SystemsPlugin)
        .add_plugin(DebugLinesPlugin)
        .add_startup_system(spawn_level_1)
        .add_startup_system_to_stage(StartupStage::PostStartup, remove_trees_from_buildings)
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
    mut trees: tree::Spawn,
    mut ritual_sites: ritual_site::Spawn,
    mut dump: dump::Spawn,
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

    let mut trees_n = 0;
    let mut boulders_n = 0;

    for y in 0..h {
        for x in 0..w {
            let v = map.get_value(x, y).abs();
            let x = x as f32;
            let z = y as f32;
            let transform = Transform::from_xyz(x + hx, 0.0, z + hz);

            if v < 0.1 {
                if fastrand::f32() < 0.3 {
                    trees.spawn(tree::Tree::new(), transform);
                    trees_n += 1;
                }
                if fastrand::f32() < 0.3 {
                    trees.spawn(tree::Tree::new(), transform);
                    trees_n += 1;
                }
            } else if v > 0.3 && 0.4 > v {
                if fastrand::f32() < 0.8 {
                    boulder.spawn(Boulder::new(Stone), transform);
                    boulders_n += 1;
                } else {
                    boulder.spawn(Boulder::new(Coal), transform);
                    boulders_n += 1;
                }
            } else if v > 0.4 && 0.45 > v {
                boulder.spawn(Boulder::new(Coal), transform);
                boulders_n += 1;
            } else if v > 0.45 && 0.47 > v {
                boulder.spawn(Boulder::new(Stone), transform);
                boulders_n += 1;
            } else if v > 0.47 && 0.49 > v {
                boulder.spawn(Boulder::new(Iron), transform);
                boulders_n += 1;
            } else if v > 0.48 && 0.53 > v && fastrand::f32() < 0.05 {
                boulder.spawn(Boulder::new(Gold), transform);
                boulders_n += 1;
            } else if v > 0.47 && 1.0 > v {
                boulder.spawn(Boulder::new(Stone), transform);
                boulders_n += 1;
            }
        }
    }

    println!("{} trees, {} boulders.", trees_n, boulders_n);

    let center = Vec3::new(7.0, 0.0, 6.0);
    let pos = |x: i32, z: i32| Vec3::new(center.x + x as f32, 0.0, center.z + z as f32);
    let at = |x: i32, z: i32| Transform::from_translation(pos(x, z));

    ground.spawn(Ground, at(0, 0));

    smithery.spawn(Smithery::new(), at(3, -2));

    imp.spawn(Imp::new(), at(0, 0));

    mage.spawn(Mage::new(), at(-1, 0))
        .insert(CameraTracking::new(0.0, 10.0, -3.0));
    camera.spawn(center);

    fireplace.spawn(Fireplace::new(), at(0, 0));

    pile.spawn(Pile::new(Thing::Iron, 10.0), at(0, 1));
    ritual_sites.spawn(
        RitualSite::new(&[(Thing::Iron, 300), (Thing::Gold, 300)]),
        at(-7, -3),
    );

    let dump1 = dump
        .spawn(
            Dump::new(),
            Transform {
                rotation: Quat::from_rotation_y(-0.28 * TAU),
                translation: center + Vec3::new(-3.5, 0.0, -3.0),
                ..Default::default()
            },
        )
        .id();
    conveyor.build_chain(&[pos(1, -1), pos(-1, -3), pos(-3, -3)], Some(dump1));
}

fn remove_trees_from_buildings(
    trees: Query<(Entity, &Transform), With<Tree>>,
    others: Query<&Transform, (With<Destructable>, Without<Tree>)>,
    mut cmds: Commands,
) {
    for (a_tree, t_tree) in trees.iter() {
        for t_other in others.iter() {
            if t_tree.translation.distance_squared(t_other.translation) < 1.0 {
                cmds.entity(a_tree).despawn_recursive();
            }
        }
    }
}
