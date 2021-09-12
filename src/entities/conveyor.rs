use std::cmp::Ordering;

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_prototype_debug_lines::DebugLines;

use crate::systems::{Destructable, FocusObject, Thing};

use super::NotGround;

#[allow(unused)]
#[derive(Clone, Copy)]
struct Item {
    thing: Thing,
    amount: f32,
    size: i32,
    pos: i32,
}

#[derive(Default)]
pub struct ConveyorBelt {
    pub marked_for_thing: Option<Thing>,
    items: Vec<Item>,
    length: i32,
    output: Option<Entity>,
}

impl ConveyorBelt {
    pub fn new() -> Self {
        Self {
            length: 100,
            ..Self::default()
        }
    }

    pub fn store(&mut self, thing: Thing, amount: f32) {
        // TODO find free space, reject if not possible
        self.marked_for_thing = Some(thing);
        self.items.insert(
            0,
            Item {
                thing,
                amount,
                size: 25,
                pos: 0,
            },
        )
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
    belts: Query<'w, 's, &'static Transform, With<ConveyorBelt>>,
    debug: ResMut<'w, DebugLines>,
}

#[derive(Clone, Copy)]
struct BeltDef(Vec3, Vec3, Vec3);

impl BeltDef {
    fn angle(&self) -> f32 {
        let v = self.2 - self.0;
        v.x.atan2(v.z)
    }
}

fn get_mids(from: Vec3, to: Vec3) -> Vec<Vec3> {
    let way = to - from;
    let dir = way.normalize();
    let steps = way.length().floor() as i32;
    (0..=steps)
        .map(move |step| {
            let step = step as f32;
            let pos = from + dir * step;
            pos
        })
        .collect()
}

#[allow(unused)]
fn debug_vec3_strip(debug: &mut DebugLines, chain: &[Vec3]) {
    let mut a = chain[0];
    for (i, b) in chain[1..].iter().enumerate() {
        debug.line_colored(
            a,
            *b,
            10.0,
            if i % 2 == 0 { Color::WHITE } else { Color::RED },
        );
        a = *b;
    }
}

fn debug_belt_defs(debug: &mut DebugLines, defs: &[BeltDef]) {
    for (i, BeltDef(a, m, b)) in defs.iter().copied().enumerate() {
        let color = if i % 2 == 0 { Color::WHITE } else { Color::RED };
        debug.line_colored(a, m, 10.0, color);
        debug.line_colored(m, b, 10.0, color);
    }
}

impl<'w, 's> ConveyorSpawn<'w, 's> {
    pub fn build_chain(&mut self, points: &[Vec3], output: Option<Entity>) {
        assert!(points.len() >= 2);
        let mut from = points[0];
        let mut chain = vec![from];

        for to in &points[1..] {
            chain.extend(&get_mids(from, *to)[1..]);
            from = *chain.last().unwrap();
        }

        //debug_vec3_strip(&mut self.debug, &chain);

        let defs: Vec<(Entity, BeltDef)> = (0..chain.len())
            .map(|index| {
                if index > 0 && index < chain.len() - 1 {
                    let before = chain[index - 1];
                    let mid = chain[index];
                    let after = chain[index + 1];
                    BeltDef(mid + 0.5 * (before - mid), mid, mid + 0.5 * (after - mid))
                } else if index == 0 {
                    let mid = chain[index];
                    let after = chain[index + 1];
                    BeltDef(mid - 0.5 * (after - mid), mid, mid + 0.5 * (after - mid))
                } else {
                    let before = chain[index - 1];
                    let mid = chain[index];
                    BeltDef(mid + 0.5 * (before - mid), mid, mid - 0.5 * (before - mid))
                }
            })
            .map(|def| (self.cmds.spawn().id(), def))
            .collect();

        self.cmds.spawn().insert(ConveyorChain {
            belts: defs.iter().map(|p| p.0).collect(),
        });

        debug_belt_defs(
            &mut self.debug,
            &defs.iter().map(|(_, def)| def).copied().collect::<Vec<_>>(),
        );

        let mut output = output;
        for (entity, def) in defs.into_iter().rev() {
            let model = self.model(self.assets.material.clone());
            self.cmds
                .entity(entity)
                .push_children(&[model])
                .insert_bundle((
                    ConveyorBelt {
                        output,
                        ..ConveyorBelt::new()
                    },
                    Transform {
                        rotation: Quat::from_rotation_y(def.angle()),
                        translation: def.1,
                        scale: Vec3::ONE,
                    },
                    GlobalTransform::identity(),
                    Destructable,
                    FocusObject,
                ));
            output = Some(entity);
        }
    }

    pub fn spawn_line<'a>(&'a mut self, from: Vec3, to: Vec3) {
        let from = self.snap_to_belt(from);
        let to = self.snap_to_belt(to);

        let mut line: Vec<_> = Self::iter_steps(from, to)
            .map(|(pos, angle)| {
                (
                    self.cmds.spawn().id(),
                    ConveyorBelt::new(),
                    Transform {
                        rotation: Quat::from_rotation_y(angle),
                        translation: pos,
                        scale: Vec3::ONE,
                    },
                )
            })
            .collect();

        for index in 0..line.len() - 1 {
            let next_entity = line[index + 1].0;
            line[index].1.output = Some(next_entity);
        }

        self.cmds.spawn().insert(ConveyorChain {
            belts: line.iter().map(|p| p.0).collect(),
        });

        for (entity, belt, transform) in line {
            let model = self.model(self.assets.material.clone());
            self.cmds
                .entity(entity)
                .insert_bundle((
                    belt,
                    transform,
                    GlobalTransform::identity(),
                    Destructable,
                    FocusObject,
                ))
                .push_children(&[model]);
        }
    }

    pub fn snap_to_belt(&self, from: Vec3) -> Vec3 {
        self.belts
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
        for (pos, angle) in Self::iter_steps(from, to) {
            self.spawn_ghost(pos, angle, parent);
        }
    }

    fn iter_steps(from: Vec3, to: Vec3) -> impl Iterator<Item = (Vec3, f32)> {
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

            let from = conveyor.snap_to_belt(from_transform.translation);
            let to = conveyor.snap_to_belt(to_transform.translation);
            conveyor.spawn_ghostline(from, to, parent);
        }
    }
}

struct ConveyorChain {
    belts: Vec<Entity>,
}

impl ConveyorBelt {
    fn left(&self) -> i32 {
        self.items.first().map(|i| i.left()).unwrap_or(self.length)
    }
}

impl Item {
    fn left(&self) -> i32 {
        self.pos - self.size / 2
    }

    fn right(&self) -> i32 {
        self.pos - self.size / 2
    }

    fn extent(&self) -> i32 {
        self.size / 2
    }
}

fn convey_items(
    chains: Query<&ConveyorChain>,
    mut query_belts: QuerySet<(QueryState<&mut ConveyorBelt>, QueryState<&ConveyorBelt>)>,
) {
    for chain in chains.iter() {
        for belt_entity in chain.belts.iter().rev().copied() {
            let belts = query_belts.q1();
            let belt = belts.get(belt_entity).unwrap();
            if belt.items.len() > 0 {
                let next_belt = belt.output.map(|e| belts.get(e).unwrap());

                if let Some(next_belt) = next_belt {
                    let next_belt_left = next_belt.left();
                    let mut belts = query_belts.q0();
                    let mut belt = belts.get_mut(belt_entity).unwrap();
                    let mut transfer_index = None;
                    let speed = 1;
                    let length = belt.length;
                    let mut right = length + next_belt_left; // note that next belt left could be negative

                    for (index, item) in belt.items.iter_mut().enumerate().rev() {
                        if item.right() < right {
                            item.pos += speed.min(right - item.pos - speed - item.extent());
                            right = item.left();

                            // if the item should belong to the next belt
                            if item.pos > length {
                                transfer_index = Some(index); // then remember the index for later copy
                                item.pos -= length; // and already set the pos to the pos in the next belt
                            }
                        }
                    }

                    if let Some(index) = transfer_index {
                        let items: Vec<Item> = belt.items.drain(index..).collect();
                        let output = belt.output.unwrap();
                        let mut next_belt = belts.get_mut(output).unwrap();
                        next_belt.items.splice(0..0, items);
                    }
                } else {
                    let mut belts = query_belts.q0();
                    let mut belt = belts.get_mut(belt_entity).unwrap();
                    let speed = 1;
                    let mut right = belt.length;

                    for item in belt.items.iter_mut().rev() {
                        if item.right() < right {
                            item.pos += speed.min(right - item.pos - speed - item.extent());
                            right = item.left();
                        }
                    }
                }
            }
        }
    }
}

fn debug_items(belts: Query<(&ConveyorBelt, &Transform)>, mut debug: ResMut<DebugLines>) {
    for (belt, transform) in belts.iter() {
        let c_pos = transform.translation;
        let c_dir = Transform::from_rotation(transform.rotation).mul_vec3(Vec3::Z);
        let c_normal = Transform::from_rotation(transform.rotation).mul_vec3(Vec3::X);
        let length_f32 = belt.length as f32;
        // ASSUMPTION: conveyor world length is 1.0, so 0.5 * c_dir is half its main direction
        let c_start = c_pos - 0.5 * c_dir;
        let c_end = c_pos + 0.5 * c_dir;

        for item in belt.items.iter() {
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
