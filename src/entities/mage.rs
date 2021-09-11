use std::f32::consts::FRAC_PI_4;

use bevy::{
    ecs::system::{EntityCommands, SystemParam},
    prelude::*,
};

use crate::systems::{Focus, Stack, Thing};

use super::{Boulder, Conveyor, NotGround, Pile};

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
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets)
            .add_system(update);
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

        let mut entity_cmds = self.cmds.spawn_bundle((
            mage,
            transform,
            GlobalTransform::identity(),
            Focus::default(),
        ));
        entity_cmds.push_children(&[model]);
        entity_cmds
    }
}

#[derive(Clone)]
pub struct MageAssets {
    pub transform: Transform,
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
}

fn load_assets(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.insert_resource(MageAssets {
        transform: Transform {
            translation: Vec3::new(0.0, 0.25, 0.05),
            rotation: Quat::from_rotation_x(0.4) * Quat::from_rotation_y(-FRAC_PI_4),
            scale: Vec3::ONE,
        },
        material: materials.add(StandardMaterial {
            base_color: Color::MIDNIGHT_BLUE,
            metallic: 0.0,
            roughness: 0.0,
            reflectance: 0.0,
            ..Default::default()
        }),
        mesh: meshes.add(
            shape::Capsule {
                latitudes: 16,
                longitudes: 8,
                ..Default::default()
            }
            .into(),
        ),
    });
}

#[derive(Default)]
pub struct MageInteractable {
    pub near: bool,
}

fn update(
    mut mages: Query<(&mut Mage, &Focus)>,
    mut boulder: Query<&mut Boulder>,
    mut pile: Query<&mut Pile>,
    mut conveyor: Query<&mut Conveyor>,
) {
    for (mut mage, focus) in mages.iter_mut() {
        if mage.interact_with_focus {
            mage.interact_with_focus = false;

            if let Some(entity) = focus.entity {
                if let Ok(mut boulder) = boulder.get_mut(entity) {
                    boulder.marked_for_digging = !boulder.marked_for_digging;
                }

                if let Ok(mut pile) = pile.get_mut(entity) {
                    if pile.amount >= 1.0 {
                        pile.amount -= 1.0;
                        mage.put_into_inventory(pile.load, 1.0);
                    }
                }

                if let Ok(mut conveyor) = conveyor.get_mut(entity) {
                    if let Some(thing) = mage.take_first(1.0) {
                        conveyor.store(thing, 1.0);
                    } else {
                        conveyor.marked_for_thing = None;
                    }
                }
            }
        }
    }
}
