use super::{NotGround, StoreIntoPile};
use crate::{assets, systems::*};
use bevy::{
    ecs::system::{EntityCommands, SystemParam},
    prelude::*,
};
use std::f32::consts::FRAC_PI_4;

#[derive(Default)]
pub struct Mage {
    pub inventory: Vec<Stack>,
    pub interact_with_focus: bool,
}

impl Mage {
    pub fn new() -> Self {
        Self {
            inventory: vec![
                Stack::default(),
                Stack::default(),
                Stack::default(),
                Stack::default(),
            ],
            ..Self::default()
        }
    }

    pub fn put_into_inventory(&mut self, thing: Thing, amount: f32) {
        for stack in self.inventory.iter_mut() {
            if stack.thing == Some(thing) {
                stack.amount += amount;
                return;
            } else if stack.thing.is_none() {
                stack.thing = Some(thing);
                stack.amount += amount;
                return;
            }
        }
    }

    pub fn peek_first(&self) -> Option<Thing> {
        for stack in self.inventory.iter() {
            if stack.amount > 0.0 {
                return stack.thing;
            }
        }

        None
    }

    pub fn take_first(&mut self, amount: f32) -> Option<Thing> {
        for stack in self.inventory.iter_mut() {
            if stack.amount >= amount {
                stack.amount -= amount;
                return stack.thing;
            }
        }

        None
    }
}

pub struct MagePlugin;

impl Plugin for MagePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets);
    }
}

#[derive(SystemParam)]
pub struct MageSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, MageAssets>,
}

impl<'w, 's> MageSpawn<'w, 's> {
    pub fn spawn<'a>(&'a mut self, mage: Mage, transform: Transform) -> EntityCommands<'w, 's, 'a> {
        let model = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: self.assets.transform.clone(),
                material: self.assets.material.clone(),
                mesh: self.assets.mesh.clone(),
                ..Default::default()
            })
            .insert(NotGround)
            .id();

        let face = self
            .cmds
            .spawn_bundle(GizmoBundle {
                transform: Transform::from_xyz(0.0, 1.0, 0.0),
                gizmo: Gizmo {
                    shape: GizmoShape::Billboard {
                        texture: Some(self.assets.face_texture.clone()),
                        size: 0.5,
                    },
                    wireframe: Color::WHITE,
                    color: Color::WHITE,
                },
                ..Default::default()
            })
            .id();

        let mut entity_cmds = self.cmds.spawn_bundle((
            mage,
            transform,
            GlobalTransform::identity(),
            Focus::default(),
        ));
        entity_cmds.push_children(&[model, face]);
        entity_cmds
    }
}

#[derive(Clone)]
pub struct MageAssets {
    pub transform: Transform,
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
    pub face_texture: Handle<Texture>,
}

fn load_assets(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: Res<AssetServer>,
) {
    cmds.insert_resource(MageAssets {
        transform: Transform {
            translation: Vec3::new(0.0, 0.25, 0.05),
            rotation: Quat::from_rotation_x(0.4) * Quat::from_rotation_y(-FRAC_PI_4),
            scale: Vec3::ONE,
        },
        material: materials.add(flat_material(Color::MIDNIGHT_BLUE)),
        mesh: meshes.add(
            shape::Capsule {
                latitudes: 16,
                longitudes: 8,
                radius: 0.2,
                depth: 0.6,
                ..Default::default()
            }
            .into(),
        ),
        face_texture: assets.load(assets::emojis::MAGE),
    });
}

impl Mage {
    pub fn try_drop_item(&mut self, transform: &Transform, cmds: &mut Commands) {
        for stack in self.inventory.iter_mut() {
            if stack.amount >= 1.0 {
                stack.amount -= 1.0;

                cmds.spawn_bundle((
                    StoreIntoPile {
                        load: stack.thing.unwrap(),
                        amount: 1.0,
                        pile: None,
                    },
                    transform.clone(),
                ));

                break;
            }
        }
    }
}
