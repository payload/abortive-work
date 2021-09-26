use bevy::{ecs::system::SystemParam, prelude::*};

pub struct Ground;

pub struct GroundModel;

pub struct GroundPlugin;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets);
    }
}

#[derive(SystemParam)]
pub struct GroundSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, GroundAssets>,
}

impl<'w, 's> GroundSpawn<'w, 's> {
    pub fn spawn(&mut self, Ground: Ground, transform: Transform) {
        let model = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: self.assets.transform.clone(),
                material: self.assets.material.clone(),
                mesh: self.assets.mesh.clone(),
                ..Default::default()
            })
            .insert(GroundModel)
            .id();

        self.cmds
            .spawn_bundle((Ground, transform, GlobalTransform::identity()))
            .push_children(&[model]);
    }
}

#[derive(Clone)]
pub struct GroundAssets {
    transform: Transform,
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
}

fn load_assets(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.insert_resource(GroundAssets {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.5, 0.4, 0.4),
            roughness: 1.0,
            reflectance: 0.0,
            metallic: 0.0,
            ..Default::default()
        }),
        mesh: meshes.add(shape::Plane { size: 300.0 }.into()),
    });
}
