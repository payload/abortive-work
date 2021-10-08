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
        let model = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: self.res.sign_transform.clone(),
                material: self.res.sign_material.clone(),
                mesh: self.res.sign_mesh.clone(),
                ..Default::default()
            })
            .insert(Model)
            .id();

        let content_model = self.cmds.spawn().id();

        self.cmds
            .spawn_bundle((
                Sign {
                    thing,
                    content_model,
                },
                Transform {
                    translation: pos,
                    rotation: Quat::from_rotation_y(10.0 * (0.5 - fastrand::f32()).to_radians()),
                    scale: Vec3::ONE,
                },
                GlobalTransform::identity(),
                Destructable,
                FocusObject::new(),
            ))
            .push_children(&[model, content_model]);
    }
}

#[derive(Clone)]
pub struct Resource {
    pub sign_transform: Transform,
    pub sign_material: Handle<StandardMaterial>,
    pub sign_mesh: Handle<Mesh>,
    pub triangle_mesh: Handle<Mesh>,
}

fn init_resource(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.insert_resource(Resource {
        sign_transform: Transform {
            translation: Vec3::new(0.0, 0.5, 0.0),
            rotation: Quat::from_rotation_x(30f32.to_radians()),
            scale: Vec3::ONE,
        },
        sign_material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.53, 0.36, 0.24),
            reflectance: 0.0,
            roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        }),
        sign_mesh: meshes.add(shape::Box::new(0.45, 0.4, 0.05).into()),

        triangle_mesh: meshes.add(triangle()),
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
            let material = materials.get(thing);
            let rot = Quat::from_rotation_x(-60f32.to_radians());

            cmds.entity(sign.content_model).insert_bundle(PbrBundle {
                transform: Transform {
                    translation: 0.5 * Vec3::Y + rot.mul_vec3(0.051 * Vec3::Y),
                    rotation: rot,
                    scale: 0.3 * Vec3::ONE,
                },
                mesh: res.triangle_mesh.clone(),
                material,
                ..Default::default()
            });
        } else if let Ok(mut visible) = visible.get_mut(sign.content_model) {
            visible.is_visible = false;
        }
    }
}
