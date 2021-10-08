use crate::systems::*;
use bevy::{ecs::system::SystemParam, prelude::*};

use super::{Blocking, NotGround};

#[derive(Default)]
pub struct Tree {
    pub mass: f32,
    pub mark_cut_tree: bool,
    pub tree_radius: f32,
}

pub struct Model;

pub struct MarkCutTree;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::Startup, init_resource)
            .add_system_to_stage(CoreStage::Last, update_trees);
    }
}

impl Tree {
    pub fn new() -> Self {
        Self {
            mass: 2.0,
            tree_radius: 0.2 - 0.12 * fastrand::f32(),
            ..Self::default()
        }
    }

    pub fn cut(&mut self, amount: f32) -> f32 {
        if amount <= self.mass {
            self.mass -= amount;
            amount
        } else {
            let mass = self.mass;
            self.mass = 0.0;
            mass
        }
    }
}

#[derive(SystemParam)]
pub struct Spawn<'w, 's> {
    cmds: Commands<'w, 's>,
    res: Res<'w, Resource>,
}

impl<'w, 's> Spawn<'w, 's> {
    pub fn spawn(&mut self, tree: Tree, transform: Transform) {
        let random_offset =
            self.res.model_offset + Vec3::new(0.5 - fastrand::f32(), 0.0, 0.5 - fastrand::f32());
        let random_angle1 = 0.1 - 0.2 * fastrand::f32();
        let random_angle2 = 0.1 - 0.2 * fastrand::f32();
        let random_rotation =
            Quat::from_rotation_z(random_angle1).mul_quat(Quat::from_rotation_x(random_angle2));
        let random_scale = Vec3::new(
            tree.tree_radius / 0.2,
            1.0 - 0.1 * fastrand::f32(),
            tree.tree_radius / 0.2,
        );

        let model = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: Transform {
                    translation: Vec3::ZERO,
                    rotation: random_rotation,
                    scale: random_scale,
                },
                material: self.res.material.clone(),
                mesh: self.res.mesh.clone(),
                ..Default::default()
            })
            .insert(NotGround)
            .insert(Blocking)
            .insert(Model)
            .id();

        self.cmds
            .spawn_bundle((
                tree,
                Transform {
                    translation: transform.translation + random_offset,
                    ..transform
                },
                GlobalTransform::identity(),
                Destructable,
                FocusObject::new(),
            ))
            .push_children(&[model]);
    }
}

#[derive(Clone)]
pub struct Resource {
    pub model_offset: Vec3,
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
}

fn init_resource(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut debug_config: ResMut<DebugConfig>,
    materials: Res<ThingMaterials>,
) {
    let capsule = shape::Capsule {
        radius: 0.2,
        depth: 1.0,
        latitudes: 5,
        longitudes: 24,
        rings: 5,
        ..Default::default()
    };
    let mesh = meshes.add(capsule.into());

    debug_config.tree_capsule_mesh = Some(mesh.clone());
    debug_config.tree_capsule = capsule;

    cmds.insert_resource(Resource {
        model_offset: Vec3::new(0.0, 0.3, 0.0),
        material: materials.get(Thing::Wood),
        mesh,
    });
}

fn update_trees(
    trees: Query<(Entity, &Tree, Option<&MarkCutTree>), Changed<Tree>>,
    mut cmds: Commands,
) {
    for (entity, tree, mark) in trees.iter() {
        if tree.mass <= 0.0 {
            cmds.entity(entity).despawn_recursive();
        }

        if tree.mark_cut_tree && mark.is_none() {
            cmds.entity(entity).insert(MarkCutTree);
        } else if !tree.mark_cut_tree && mark.is_some() {
            cmds.entity(entity).remove::<MarkCutTree>();
        }
    }
}
