use std::cmp::Ordering;

use bevy::{
    ecs::system::{EntityCommands, SystemParam},
    prelude::*,
};
use bevy_prototype_debug_lines::DebugLines;

use crate::systems::{Destructable, FocusObject, Thing};

use super::NotGround;

struct Item {
    thing: Thing,
    amount: f32,
    size: i32,
    pos: i32,
}

#[derive(Default)]
pub struct Conveyor {
    pub marked_for_thing: Option<Thing>,
    items: Vec<Item>,
    length: i32,
}

impl Conveyor {
    pub fn new() -> Self {
        Self {
            length: 100,
            ..Self::default()
        }
    }

    pub fn store(&mut self, thing: Thing, amount: f32) {
        self.marked_for_thing = Some(thing);
        self.items.push(Item {
            thing,
            amount,
            size: 25,
            pos: 0,
        })
    }
}

pub struct ConveyorPlugin;

impl Plugin for ConveyorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets)
            .add_system_to_stage(CoreStage::Update, update_lines)
            .add_system_to_stage(CoreStage::PreUpdate, convey_items)
            .add_system_to_stage(CoreStage::PostUpdate, debug_items);
    }
}

#[derive(SystemParam)]
pub struct ConveyorSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, ConveyorAssets>,
    conveyors: Query<'w, 's, &'static Transform, With<Conveyor>>,
}

impl<'w, 's> ConveyorSpawn<'w, 's> {
    pub fn spawn_line<'a>(&'a mut self, from: Vec3, to: Vec3) {
        let from = self.snap_to_conveyor(from);
        let to = self.snap_to_conveyor(to);

        for (pos, angle) in Self::spawn_positions_at_line(from, to) {
            self.spawn(
                Conveyor::new(),
                Transform {
                    rotation: Quat::from_rotation_y(angle),
                    translation: pos,
                    ..Default::default()
                },
            );
        }
    }

    pub fn snap_to_conveyor(&self, from: Vec3) -> Vec3 {
        self.conveyors
            .iter()
            .map(|t| (t.translation, t.translation.distance_squared(from)))
            .filter(|(_, d)| *d < 1.0)
            .min_by(|(_, a), (_, b)| {
                if a < b {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })
            .map(|(pos, _)| pos)
            .unwrap_or(from)
    }

    pub fn ghostline_from_point_to_entity(&mut self, from: Vec3, to: Entity) -> Entity {
        self.cmds
            .spawn_bundle((
                LineFrom { to },
                Transform::from_translation(from),
                GlobalTransform::identity(),
            ))
            .id()
    }

    pub fn spawn_ghostline(&mut self, from: Vec3, to: Vec3, parent: Entity) {
        for (pos, angle) in Self::spawn_positions_at_line(from, to) {
            self.spawn_ghost(pos, angle, parent);
        }
    }

    fn spawn_positions_at_line(from: Vec3, to: Vec3) -> impl Iterator<Item = (Vec3, f32)> {
        let way = to - from;
        let dir = way.normalize();
        let angle = dir.x.atan2(dir.z);
        let steps = way.length().floor() as i32;
        (0..=steps).map(move |step| {
            let step = step as f32;
            let pos = from + dir * step;
            (pos, angle)
        })
    }

    fn spawn_ghost(&mut self, pos: Vec3, angle: f32, parent: Entity) {
        let model = self.model(self.assets.ghost_material.clone());
        let ghost = self
            .cmds
            .spawn_bundle((
                GlobalTransform::identity(),
                Transform {
                    rotation: Quat::from_rotation_y(angle),
                    translation: pos,
                    ..Default::default()
                },
            ))
            .push_children(&[model])
            .id();
        self.cmds.entity(parent).push_children(&[ghost]);
    }

    fn model(&mut self, material: Handle<StandardMaterial>) -> Entity {
        self.cmds
            .spawn_bundle(PbrBundle {
                material,
                mesh: self.assets.mesh.clone(),
                transform: self.assets.transform.clone(),
                ..Default::default()
            })
            .insert(NotGround)
            .id()
    }

    fn spawn<'a>(
        &'a mut self,
        conveyor: Conveyor,
        transform: Transform,
    ) -> EntityCommands<'w, 's, 'a> {
        let model = self.model(self.assets.material.clone());
        let mut entity_cmds = self.cmds.spawn_bundle((
            conveyor,
            transform,
            GlobalTransform::identity(),
            Destructable,
            FocusObject,
        ));
        entity_cmds.push_children(&[model]);
        entity_cmds
    }
}

#[derive(Clone)]
pub struct ConveyorAssets {
    pub transform: Transform,
    pub material: Handle<StandardMaterial>,
    pub ghost_material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
}

fn load_assets(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.insert_resource(ConveyorAssets {
        transform: Transform::from_xyz(0.0, 0.25, 0.0),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.1, 0.1, 0.1),
            metallic: 0.0,
            roughness: 0.0,
            reflectance: 0.0,
            ..Default::default()
        }),
        ghost_material: materials.add(StandardMaterial {
            base_color: Color::rgba(0.1, 0.1, 0.1, 0.4),
            metallic: 0.0,
            roughness: 0.0,
            reflectance: 0.0,
            ..Default::default()
        }),
        mesh: meshes.add(shape::Box::new(0.7, 0.1, 0.95).into()),
    });
}

struct LineFrom {
    to: Entity,
}

struct LineParent;

fn update_lines(
    mut frame: Local<usize>,
    mut cmds: Commands,
    mut conveyor: ConveyorSpawn,
    ghostlines: Query<(&Transform, &LineFrom)>,
    transforms: Query<&Transform>,
    parents: Query<Entity, With<LineParent>>,
) {
    if *frame < 2 {
        *frame += 1;
        return;
    }

    *frame = 0;

    for parent in parents.iter() {
        cmds.entity(parent).despawn_recursive();
    }

    for (from_transform, ghostline) in ghostlines.iter() {
        if let Ok(to_transform) = transforms.get(ghostline.to) {
            let parent = cmds
                .spawn_bundle((
                    LineParent,
                    Transform::identity(),
                    GlobalTransform::identity(),
                ))
                .id();

            let from = conveyor.snap_to_conveyor(from_transform.translation);
            let to = conveyor.snap_to_conveyor(to_transform.translation);
            conveyor.spawn_ghostline(from, to, parent);
        }
    }
}

fn convey_items(mut conveyors: Query<&mut Conveyor>) {
    for mut conveyor in conveyors.iter_mut() {
        let length = conveyor.length;

        for mut item in conveyor.items.iter_mut() {
            item.pos += 1;
            if item.pos >= length {
                item.pos = item.pos % length;
            }
        }
    }
}

fn debug_items(conveyors: Query<(&Conveyor, &Transform)>, mut debug: ResMut<DebugLines>) {
    for (conveyor, transform) in conveyors.iter() {
        let c_pos = transform.translation;
        let c_dir = Transform::from_rotation(transform.rotation).mul_vec3(Vec3::Z);
        let c_normal = Transform::from_rotation(transform.rotation).mul_vec3(Vec3::X);
        let length_f32 = conveyor.length as f32;
        // ASSUMPTION: conveyor world length is 1.0, so 0.5 * c_dir is half its main direction
        let c_start = c_pos - 0.5 * c_dir;
        let c_end = c_pos + 0.5 * c_dir;

        for item in conveyor.items.iter() {
            let item_size = item.size as f32 / length_f32;
            let half_length = 0.5 * item_size * c_dir;
            let half_width = 0.5 * item_size * c_normal;
            let linear_item_pos = item.pos as f32 / length_f32;
            let item_pos = c_start + linear_item_pos * (c_end - c_start);
            let start = item_pos - half_length;
            let end = item_pos + half_length;

            debug.line_colored(start - half_width, start + half_width, 0.0, Color::YELLOW);
            debug.line_colored(end - half_width, end + half_width, 0.0, Color::YELLOW);
            debug.line_colored(start - half_width, end - half_width, 0.0, Color::YELLOW);
            debug.line_colored(start + half_width, end + half_width, 0.0, Color::YELLOW);
        }
    }
}
