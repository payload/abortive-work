use bevy::{
    ecs::system::{EntityCommands, SystemParam},
    prelude::*,
};

use crate::systems::AugmentSpawn;

use super::NotGround;

#[derive(Default)]
pub struct Mage {}

impl Mage {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct MagePlugin;

impl Plugin for MagePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets)
            .add_system(update_mage_interactables);
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

        let mut entity_cmds =
            self.cmds
                .spawn_bundle((mage, transform, GlobalTransform::identity()));
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
        transform: Transform::from_xyz(0.0, 0.25, 0.0),
        material: materials.add(StandardMaterial {
            base_color: Color::MIDNIGHT_BLUE,
            metallic: 0.0,
            roughness: 0.0,
            reflectance: 0.0,
            ..Default::default()
        }),
        mesh: meshes.add(
            shape::Capsule {
                latitudes: 2,
                longitudes: 4,
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

fn update_mage_interactables(
    mages: Query<(&Transform, &Mage)>,
    mut interactables: Query<(Entity, &Transform, &mut MageInteractable)>,
    mut augment: AugmentSpawn,
) {
    let mut vec: Vec<_> = interactables
        .iter_mut()
        .map(|p| (p.0, p.1, p.2, false))
        .collect();

    for (mage_transform, _mage) in mages.iter() {
        let m_pos = mage_transform.translation;

        for (_, t, _, near) in vec.iter_mut() {
            if m_pos.distance_squared(t.translation) < 4.0 {
                *near = true;
            }
        }
    }

    for (e, _, act, near) in vec.iter_mut() {
        if *near && !act.near {
            act.near = true;
            augment.add_coin(*e);
        } else if !*near && act.near {
            act.near = false;
            augment.remove_coin(*e);
        }
    }
}
