use crate::systems::*;
use bevy::{ecs::system::SystemParam, prelude::*};

use super::{Blocking, NotGround};

#[derive(Default)]
pub struct Component {
    pub mass: f32,
    pub mark_cut_tree: bool,
}

pub struct Model;

pub struct MarkCutTree;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, init_resource)
            .add_system_to_stage(CoreStage::Last, update_trees);
    }
}

impl Component {
    pub fn new() -> Self {
        Self {
            mass: 2.0,
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
    pub fn spawn(&mut self, component: Component, transform: Transform) {
        let random_offset = self.res.model_offset
            + Vec3::new(
                0.1 - 0.2 * fastrand::f32(),
                0.0,
                0.1 - 0.2 * fastrand::f32(),
            );
        let random_angle1 = 0.1 - 0.2 * fastrand::f32();
        let random_angle2 = 0.1 - 0.2 * fastrand::f32();
        let random_rotation =
            Quat::from_rotation_z(random_angle1).mul_quat(Quat::from_rotation_x(random_angle2));
        let random_scale = Vec3::new(
            1.0 - 0.2 * fastrand::f32(),
            1.0 - 0.1 * fastrand::f32(),
            1.0 - 0.2 * fastrand::f32(),
        );

        let model = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: Transform {
                    translation: random_offset,
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
                component,
                transform,
                GlobalTransform::identity(),
                Destructable,
                FocusObject,
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
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.insert_resource(Resource {
        model_offset: Vec3::new(0.0, 0.3, 0.0),
        material: materials.add(StandardMaterial {
            base_color: Color::DARK_GREEN,
            reflectance: 0.0,
            roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        }),
        mesh: meshes.add(
            shape::Capsule {
                radius: 0.1,
                ..Default::default()
            }
            .into(),
        ),
    });
}

fn update_trees(
    trees: Query<(Entity, &Component, Option<&MarkCutTree>), Changed<Component>>,
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
