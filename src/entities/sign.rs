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
            .add_system_to_stage(CoreStage::First, display_content);
    }
}

#[derive(SystemParam)]
pub struct Spawn<'w, 's> {
    cmds: Commands<'w, 's>,
    res: Res<'w, Resource>,
}

impl<'w, 's> Spawn<'w, 's> {
    pub fn spawn(&mut self, thing: Option<Thing>, pos: Vec3) {
        let mut transform = self.res.transform.clone();
        transform.rotate(Quat::from_rotation_y(
            10.0 * (0.5 - fastrand::f32()).to_radians(),
        ));
        let model = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform,
                material: self.res.material.clone(),
                mesh: self.res.mesh.clone(),
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
                Transform::from_translation(pos),
                GlobalTransform::identity(),
                Destructable,
                FocusObject::new(),
            ))
            .push_children(&[model, content_model]);
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
) {
    cmds.insert_resource(Resource {
        transform: Transform {
            translation: Vec3::new(0.0, 0.5, 0.0),
            rotation: Quat::from_rotation_x(30f32.to_radians()),
            scale: Vec3::ONE,
        },
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.53, 0.36, 0.24),
            reflectance: 0.0,
            roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        }),
        mesh: meshes.add(shape::Box::new(0.45, 0.4, 0.05).into()),
    });
}

fn display_content(
    signs: Query<&Sign, Changed<Sign>>,
    mut visible: Query<&mut Visible>,
    mut _cmds: Commands,
) {
    for sign in signs.iter() {
        if let Some(thing) = sign.thing {
            match thing {
                // TODO insert PbrBundle
                Thing::Stone => {}
                Thing::Coal => {}
                Thing::Iron => {}
                Thing::Gold => {}
                Thing::Tool => {}
                Thing::Wood => {}
            }
        } else if let Ok(mut visible) = visible.get_mut(sign.content_model) {
            visible.is_visible = false;
        }
    }
}
