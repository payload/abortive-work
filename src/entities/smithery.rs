use bevy::{ecs::system::SystemParam, prelude::*};
use crate::systems::*;

pub struct Smithery;

pub struct SmitheryPlugin;

impl Plugin for SmitheryPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_smithery_assets);
    }
}

#[derive(SystemParam)]
pub struct SmitherySpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, SmitheryAssets>,
    time: Res<'w, Time>,
}

impl<'w, 's> SmitherySpawn<'w, 's> {
    pub fn spawn(&mut self, smithery: Smithery, transform: Transform) {
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
            .spawn_bundle((
                smithery,
                transform,
                GlobalTransform::identity(),
                FunnyAnimation {
                    offset: self.time.seconds_since_startup().fract() as f32,
                },
            ))
            .push_children(&[model]);
    }
}

#[derive(Clone)]
pub struct SmitheryAssets {
    transform: Transform,
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
}

fn load_smithery_assets(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.insert_resource(SmitheryAssets {
        transform: Transform::from_xyz(0.0, 0.4, 0.0),
        material: materials.add(StandardMaterial {
            unlit: true,
            base_color: Color::DARK_GRAY,
            ..Default::default()
        }),
        mesh: meshes.add(shape::Box::new(1.2, 0.8, 1.2).into()),
    });
}
