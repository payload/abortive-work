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

mod assets;
mod extensions;

use crate::entities::{
    dump::Dump, generator::Generator, ritual_site::RitualSite, transformer::Transformer,
};

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
        .add_startup_system_to_stage(StartupStage::PostStartup, spawn_level_1)
        .add_system_to_stage(CoreStage::PostUpdate, remove_trees_from_buildings)
        .run();
}

fn spawn_level_1(
    mut ground: GroundSpawn,
    mut boulder: BoulderSpawn,
    mut imp: ImpSpawn,
    mut mage: MageSpawn,
    mut camera: CameraSpawn,
    mut pile: PileSpawn,
    mut conveyor: ConveyorSpawn,
    mut trees: tree::Spawn,
    mut ritual_sites: ritual_site::Spawn,
    mut dump: dump::Spawn,
    mut generator: generator::Spawn,
    mut transformer: transformer::Spawn,
    mut sign: sign::Spawn,
) {
    use Thing::*;

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

    imp.spawn(Imp::new(), at(0, 0));

    mage.spawn(Mage::new(), at(-1, 0))
        .insert(CameraTracking::new(0.0, 10.0, -3.0));
    camera.spawn(center);

    pile.spawn(Pile::new(Iron, 10.0), at(0, 1));
    ritual_sites.spawn(RitualSite::new(&[(Iron, 300), (Gold, 300)]), at(-7, -6));

    let dump1 = dump
        .spawn(
            Dump::new(),
            at(-9, -3).with_rotation(Quat::from_rotation_y(95.0f32.to_radians())),
        )
        .id();
    let dump2 = dump
        .spawn(
            Dump::new(),
            at(-9, 0).with_rotation(Quat::from_rotation_y(90.0f32.to_radians())),
        )
        .id();

    let generator1 = generator
        .spawn(
            Generator::new(Gold, 0.7),
            at(0, -1).with_rotation(Quat::from_rotation_y(90.0f32.to_radians())),
        )
        .id();

    let transformer1 = transformer.spawn(
        Transformer::new(),
        at(-5, -3).with_rotation(Quat::from_rotation_y(125.0f32.to_radians())),
    );

    conveyor.spawn_chain_over(
        ChainLink::Entity(generator1),
        ChainLink::Entity(transformer1.input_belt),
        &[center + Vec3::new(1.0, 0.0, -4.0), pos(2, -9), pos(-3, -7)],
    );
    conveyor.spawn_chain(
        ChainLink::Entity(transformer1.output_belt1),
        ChainLink::Entity(dump1),
    );
    conveyor.spawn_chain(
        ChainLink::Entity(transformer1.output_belt2),
        ChainLink::Entity(dump2),
    );

    sign.spawn(Some(Stone), pos(2, -4));
    sign.spawn(Some(Stone), pos(0, 2));

    // to prevent unused ChainLink::Pos
    conveyor.spawn_chain(ChainLink::Pos(pos(-30, -30)), ChainLink::Pos(pos(-40, -30)));
}

fn remove_trees_from_buildings(
    trees: Query<(Entity, &Transform), With<Tree>>,
    others: Query<&Transform, (With<Destructable>, Without<Tree>)>,
    mut cmds: Commands,
    mut frame: Local<usize>,
) {
    *frame += 1;
    if *frame == 2 {
        for (a_tree, t_tree) in trees.iter() {
            for t_other in others.iter() {
                if t_tree.translation.distance_squared(t_other.translation) < 4.0 {
                    cmds.entity(a_tree).despawn_recursive();
                }
            }
        }
    }
}
