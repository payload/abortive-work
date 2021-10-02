use crate::systems::*;
use bevy::{
    ecs::system::{EntityCommands, SystemParam},
    prelude::*,
};

use super::{BeltDef, ConveyorBelt};

#[derive(Default)]
pub struct Generator {
    next_item_time: f64,
    time_for_item: f64,
    thing: Option<Thing>,
}

pub struct Model;
pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, init_resource)
            .add_system(generate_items);
    }
}

impl Generator {
    pub fn new(thing: Thing, time_for_item: f64) -> Self {
        Self {
            time_for_item,
            thing: Some(thing),
            ..Self::default()
        }
    }
}

#[derive(SystemParam)]
pub struct Spawn<'w, 's> {
    cmds: Commands<'w, 's>,
    res: Res<'w, Resource>,
}

impl<'w, 's> Spawn<'w, 's> {
    pub fn spawn<'a>(
        &'a mut self,
        component: Generator,
        transform: Transform,
    ) -> EntityCommands<'w, 's, 'a> {
        let hole = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: self.res.hole_transform.clone(),
                material: self.res.hole_material.clone(),
                mesh: self.res.hole_mesh.clone(),
                ..Default::default()
            })
            .id();

        let model = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: self.res.transform.clone(),
                material: self.res.material.clone(),
                mesh: self.res.mesh.clone(),
                ..Default::default()
            })
            .insert(Model)
            .push_children(&[hole])
            .id();

        let pos = transform.translation;
        let start = pos;
        let mid = pos;
        let end = pos + 0.5 * Vec3::X;
        let mut e_cmds = self.cmds.spawn_bundle((
            component,
            transform,
            ConveyorBelt::new(50, BeltDef(start, mid, end)),
            GlobalTransform::identity(),
            Destructable,
            FocusObject::new(),
        ));
        e_cmds.push_children(&[model]);
        e_cmds
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
            base_color: Color::INDIGO,
            reflectance: 0.0,
            roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        }),
        hole_material: materials.add(StandardMaterial {
            base_color: Color::NAVY,
            reflectance: 0.0,
            roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        }),
        mesh: meshes.add(shape::Box::new(1.0, 0.2, 1.0).into()),
        hole_mesh: meshes.add(shape::Plane { size: 0.5 }.into()),
    });
}

fn generate_items(time: Res<Time>, mut generators: Query<(&mut ConveyorBelt, &mut Generator)>) {
    for (mut belt, mut generator) in generators.iter_mut() {
        if time.seconds_since_startup() >= generator.next_item_time {
            if let Some(thing) = generator.thing {
                generator.next_item_time = time.seconds_since_startup() + generator.time_for_item;
                belt.put_thing(thing);
            }
        }
    }
}
