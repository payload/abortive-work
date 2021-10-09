use crate::systems::*;
use bevy::{ecs::system::SystemParam, prelude::*};

pub struct Sign {
    pub thing: Option<Thing>,
    pub content_model: Entity,
}
pub struct Model;
pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, init_resource)
            .add_system_to_stage(CoreStage::PreUpdate, display_content);
    }
}

#[derive(SystemParam)]
pub struct Spawn<'w, 's> {
    cmds: Commands<'w, 's>,
    res: Res<'w, Resource>,
}

impl<'w, 's> Spawn<'w, 's> {
    pub fn spawn(&mut self, thing: Option<Thing>, pos: Vec3) {
        let content_model = self.cmds.spawn().id();
        let model = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: self.res.sign_transform.clone(),
                material: self.res.sign_material.clone(),
                mesh: self.res.sign_mesh.clone(),
                ..Default::default()
            })
            .insert(Model)
            .push_children(&[content_model])
            .id();

        self.cmds
            .spawn_bundle((
                Sign {
                    thing,
                    content_model,
                },
                Transform::from_translation(pos),
                GlobalTransform::identity(),
                Destructable,
                FocusObject::new(),
            ))
            .push_children(&[model]);
    }
}

#[derive(Clone)]
pub struct Resource {
    pub sign_transform: Transform,
    pub sign_material: Handle<StandardMaterial>,
    pub sign_mesh: Handle<Mesh>,
    pub item_mesh: Handle<Mesh>,
}

fn init_resource(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.insert_resource(Resource {
        sign_transform: Transform::from_xyz(0.0, 0.5, 0.0),
        sign_material: materials.add(flat_material(Color::AQUAMARINE)),
        sign_mesh: meshes.add(ring(0.175, 0.125, 16)),
        item_mesh: meshes.add(disk(0.125, 16)),
    });
}

fn display_content(
    signs: Query<&Sign, Changed<Sign>>,
    mut visible: Query<&mut Visible>,
    mut cmds: Commands,
    res: Res<Resource>,
    materials: Res<ThingMaterials>,
) {
    for sign in signs.iter() {
        if let Some(thing) = sign.thing {
            cmds.entity(sign.content_model).insert_bundle(PbrBundle {
                mesh: res.item_mesh.clone(),
                material: materials.get(thing),
                ..Default::default()
            });
        } else if let Ok(mut visible) = visible.get_mut(sign.content_model) {
            visible.is_visible = false;
        }
    }
}
