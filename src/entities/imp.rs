use bevy::{ecs::system::SystemParam, prelude::*};

pub struct Imp;

#[derive(Clone)]
pub struct ImpAssets {
    transform: Transform,
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
}

pub struct ImpPlugin;

impl Plugin for ImpPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets);
    }
}

#[derive(SystemParam)]
pub struct ImpSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, ImpAssets>,
}

impl<'w, 's> ImpSpawn<'w, 's> {
    pub fn spawn(&mut self, imp: Imp, transform: Transform) {
        let pbr_bundle = PbrBundle {
            transform: self.assets.transform.clone(),
            material: self.assets.material.clone(),
            mesh: self.assets.mesh.clone(),
            ..Default::default()
        };

        self.cmds
            .spawn_bundle((imp, transform, GlobalTransform::identity()))
            .with_children(|p| {
                p.spawn_bundle(pbr_bundle);
            });
    }
}

fn load_assets(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.insert_resource(ImpAssets {
        transform: Transform::from_xyz(0.0, 0.3, 0.0),
        material: materials.add(material(Color::SALMON)),
        mesh: meshes.add(shape::Box::new(0.4, 0.6, 0.4).into()),
    });

    fn material(color: Color) -> StandardMaterial {
        StandardMaterial {
            unlit: true,
            base_color: color,
            ..Default::default()
        }
    }
}
