use bevy::{
    ecs::system::{EntityCommands, SystemParam},
    prelude::*,
};

use crate::systems::disk;

use super::{MageInteractable, NotGround};

#[derive(Default)]
pub struct Fireplace {}

impl Fireplace {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct FireplacePlugin;

impl Plugin for FireplacePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets);
    }
}

#[derive(SystemParam)]
pub struct FireplaceSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, FireplaceAssets>,
}

impl<'w, 's> FireplaceSpawn<'w, 's> {
    pub fn spawn<'a>(
        &'a mut self,
        fireplace: Fireplace,
        transform: Transform,
    ) -> EntityCommands<'w, 's, 'a> {
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
            fireplace,
            transform,
            GlobalTransform::identity(),
            MageInteractable::default(),
        ));
        entity_cmds.push_children(&[model]);
        entity_cmds
    }
}

#[derive(Clone)]
pub struct FireplaceAssets {
    pub transform: Transform,
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
}

fn load_assets(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.insert_resource(FireplaceAssets {
        transform: Transform::from_xyz(0.0, 0.001, 0.0),
        material: materials.add(StandardMaterial {
            base_color: Color::MAROON,
            metallic: 0.0,
            roughness: 0.0,
            reflectance: 0.0,
            ..Default::default()
        }),
        mesh: meshes.add(disk(0.3, 24)),
    });
}
