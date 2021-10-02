use crate::systems::*;
use bevy::{ecs::system::SystemParam, prelude::*};

use super::{BeltDef, ConveyorBelt};

#[derive(Default)]
pub struct Transformer {}

pub struct Model;
pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, init_resource)
            .add_system(transform_items);
    }
}

impl Transformer {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }
}

#[derive(SystemParam)]
pub struct Spawn<'w, 's> {
    cmds: Commands<'w, 's>,
    res: Res<'w, Resource>,
}

#[derive(Clone)]
pub struct Entities {
    pub building: Entity,
    pub input_belt: Entity,
    pub output_belt: Entity,
    pub model: Entity,
}

impl<'w, 's> Spawn<'w, 's> {
    pub fn spawn(&mut self, component: Transformer, transform: Transform) -> Entities {
        let model = self.model();

        let pos = transform.translation;
        let z = transform.rotation.mul_vec3(Vec3::Z);
        let input_belt = BeltDef(pos + 0.5 * z, pos + 0.25 * z, pos + 0.25 * z);
        let output_belt = BeltDef(pos - 0.25 * z, pos - 0.25 * z, pos - 0.5 * z);

        let input_belt = self
            .cmds
            .spawn()
            .insert(ConveyorBelt::new(25, input_belt))
            .id();
        let output_belt = self
            .cmds
            .spawn()
            .insert(ConveyorBelt::new(25, output_belt))
            .id();

        let mut e_cmds = self.cmds.spawn();
        let building = e_cmds.id();
        let entities = Entities {
            building,
            input_belt,
            output_belt,
            model,
        };

        e_cmds
            .insert_bundle((
                component,
                transform,
                entities.clone(),
                GlobalTransform::identity(),
                Destructable,
                FocusObject::new(),
            ))
            .push_children(&[model, input_belt, output_belt]);

        entities
    }

    fn model(&mut self) -> Entity {
        let hole = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: self.res.hole_transform.clone(),
                material: self.res.hole_material.clone(),
                mesh: self.res.hole_mesh.clone(),
                ..Default::default()
            })
            .id();

        self.cmds
            .spawn_bundle(PbrBundle {
                transform: self.res.transform.clone(),
                material: self.res.material.clone(),
                mesh: self.res.mesh.clone(),
                ..Default::default()
            })
            .insert(Model)
            .push_children(&[hole])
            .id()
    }
}

#[derive(Clone)]
pub struct Resource {
    pub transform: Transform,
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
    pub hole_transform: Transform,
    pub hole_mesh: Handle<Mesh>,
    pub hole_material: Handle<StandardMaterial>,
}

fn init_resource(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.insert_resource(Resource {
        transform: Transform::from_xyz(0.0, 0.1, 0.0),
        hole_transform: Transform::from_xyz(0.0, 0.102, 0.1),
        material: materials.add(StandardMaterial {
            base_color: Color::GOLD,
            reflectance: 0.0,
            roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        }),
        hole_material: materials.add(StandardMaterial {
            base_color: Color::YELLOW,
            reflectance: 0.0,
            roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        }),
        mesh: meshes.add(shape::Box::new(1.0, 0.2, 1.0).into()),
        hole_mesh: meshes.add(shape::Plane { size: 0.5 }.into()),
    });
}

fn transform_items(
    mut belts: Query<&mut ConveyorBelt>,
    transformers: Query<(&Transformer, &Entities)>,
) {
    for (_transformer, entities) in transformers.iter() {
        let things = belts
            .get_mut(entities.input_belt)
            .map(|mut belt| belt.drain_items_after_pos(10));

        if let Ok(things) = things {
            if !things.is_empty() {
                if let Ok(mut belt) = belts.get_mut(entities.output_belt) {
                    for thing in things {
                        belt.force_insert_thing(thing, 0);
                    }
                }
            }
        }
    }
}
