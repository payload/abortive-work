use std::f32::consts::PI;

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::systems::Destructable;

use super::{Blocking, MageInteractable, NotGround};

#[derive(Clone)]
pub struct Boulder {
    pub material: BoulderMaterial,
    pub marked_for_digging: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoulderMaterial {
    Stone,
    Coal,
    Iron,
    Gold,
}

pub struct BoulderModel;

impl Boulder {
    pub fn new(material: BoulderMaterial) -> Self {
        Self {
            material,
            marked_for_digging: false,
        }
    }
}

#[derive(Clone)]
pub struct BoulderAssets {
    pub transform: Transform,
    pub stone: Handle<StandardMaterial>,
    pub coal: Handle<StandardMaterial>,
    pub iron: Handle<StandardMaterial>,
    pub gold: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
}

pub struct BoulderPlugin;

impl Plugin for BoulderPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_boulder_assets);
    }
}

#[derive(SystemParam)]
pub struct BoulderSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, BoulderAssets>,
}

impl<'w, 's> BoulderSpawn<'w, 's> {
    pub fn spawn(&mut self, boulder: Boulder, mut transform: Transform) {
        let mut deeper_transform = self.assets.transform.clone();
        deeper_transform.translation.y -= 0.2 * fastrand::f32();

        transform.rotate(
            Quat::from_rotation_z(0.125 * PI * (0.5 - fastrand::f32()))
                .mul_quat(Quat::from_rotation_x(0.125 * PI * (0.5 - fastrand::f32()))),
        );

        let model = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: deeper_transform,
                material: match boulder.material {
                    BoulderMaterial::Stone => self.assets.stone.clone(),
                    BoulderMaterial::Coal => self.assets.coal.clone(),
                    BoulderMaterial::Iron => self.assets.iron.clone(),
                    BoulderMaterial::Gold => self.assets.gold.clone(),
                },
                mesh: self.assets.mesh.clone(),
                ..Default::default()
            })
            .insert(NotGround)
            .insert(Blocking)
            .insert(BoulderModel)
            .id();

        self.cmds
            .spawn_bundle((
                boulder,
                transform,
                GlobalTransform::identity(),
                MageInteractable::default(),
                Destructable,
            ))
            .push_children(&[model]);
    }
}

fn load_boulder_assets(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.insert_resource(BoulderAssets {
        transform: Transform::from_xyz(0.0, 0.9, 0.0),
        stone: materials.add(material(Color::DARK_GRAY)),
        coal: materials.add(material(Color::BLACK)),
        gold: materials.add(material(Color::GOLD)),
        iron: materials.add(material(Color::ORANGE_RED)),
        mesh: meshes.add(shape::Box::new(1.0, 2.0, 1.0).into()),
    });

    fn material(color: Color) -> StandardMaterial {
        StandardMaterial {
            base_color: color,
            ..Default::default()
        }
    }
}
