use bevy::{ecs::system::SystemParam, prelude::*};

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
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets);
    }
}

#[derive(SystemParam)]
pub struct MageSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, MageAssets>,
}

impl<'w, 's> MageSpawn<'w, 's> {
    pub fn spawn(&mut self, mage: Mage, transform: Transform) -> Entity {
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

        self.cmds
            .spawn_bundle((mage, transform, GlobalTransform::identity()))
            .push_children(&[model])
            .id()
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
