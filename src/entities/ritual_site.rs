use crate::systems::*;
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(Default)]
pub struct RitualSite {}
pub struct Model;
pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, init_resource);
    }
}

impl RitualSite {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }
}

#[derive(SystemParam)]
pub struct Spawn<'w, 's> {
    cmds: Commands<'w, 's>,
    res: Res<'w, Resource>,
}

impl<'w, 's> Spawn<'w, 's> {
    pub fn spawn(&mut self, component: RitualSite, transform: Transform) {
        let model = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: self.res.transform.clone(),
                material: self.res.material.clone(),
                mesh: self.res.mesh.clone(),
                ..Default::default()
            })
            .insert(Model)
            .id();

        self.cmds
            .spawn_bundle((
                component,
                transform,
                GlobalTransform::identity(),
                Destructable,
            ))
            .push_children(&[model]);
    }
}

#[derive(Clone)]
pub struct Resource {
    pub transform: Transform,
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
}

fn init_resource(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: ResMut<AssetServer>,
) {
    cmds.insert_resource(Resource {
        transform: Transform {
            rotation: Quat::IDENTITY,
            translation: Vec3::new(0.0, 0.005, 0.0),
            scale: Vec3::ONE,
        },
        material: materials.add(StandardMaterial {
            base_color_texture: Some(assets.load("nebra4.jpg")),
            unlit: true,
            ..Default::default()
        }),
        mesh: meshes.add(disk(1.0, 24)),
    });
}
