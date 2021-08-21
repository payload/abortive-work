use bevy::{ecs::system::SystemParam, prelude::*};

pub struct Storage;

impl Storage {
    pub fn new() -> Self {
        Self
    }
}

pub struct StoragePlugin;

impl Plugin for StoragePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets);
    }
}

#[derive(SystemParam)]
pub struct StorageSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, StorageAssets>,
}

impl<'w, 's> StorageSpawn<'w, 's> {
    pub fn spawn(&mut self, Storage: Storage, transform: Transform) {
        let model = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: self.assets.transform.clone(),
                material: self.assets.material.clone(),
                mesh: self.assets.mesh.clone(),
                ..Default::default()
            })
            .id();

        self.cmds
            .spawn_bundle((Storage, transform, GlobalTransform::identity()))
            .push_children(&[model]);
    }
}

#[derive(Clone)]
pub struct StorageAssets {
    transform: Transform,
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
}

fn load_assets(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.insert_resource(StorageAssets {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        material: materials.add(StandardMaterial {
            unlit: true,
            base_color: Color::GRAY,
            ..Default::default()
        }),
        mesh: meshes.add(shape::Plane { size: 1.0 }.into()),
    });
}
