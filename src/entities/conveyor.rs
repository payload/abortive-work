use std::cmp::Ordering;

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_prototype_debug_lines::DebugLines;
use lyon::geom::CubicBezierSegment;

use super::NotGround;
use crate::extensions::{ToPoint, ToVec3};
use crate::systems::*;

#[allow(unused)]
#[derive(Clone, Copy)]
struct Item {
    thing: Thing,
    amount: f32,
    size: i32,
    pos: i32,
    displacement: f32,
}

#[derive(Default)]
pub struct ConveyorBelt {
    pub marked_for_thing: Option<Thing>,
    items: Vec<Item>,
    length: i32,
    pub output: Option<Entity>,
    pub def: BeltDef,
}

impl ConveyorBelt {
    pub fn new(length: i32, def: BeltDef) -> Self {
        Self {
            length,
            def,
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
                displacement: fastrand::f32() - 0.5,
            },
        )
    }

    pub fn put_thing(&mut self, thing: Thing) {
        for pos in self.free_pos(25) {
            self.items.insert(
                0,
                Item {
                    thing,
                    amount: 1.0,
                    size: 25,
                    pos: pos,
                    displacement: fastrand::f32() - 0.5,
                },
            );
        }
    }

    pub fn force_insert_thing(&mut self, thing: Thing, pos: i32) {
        let item = Item {
            thing,
            amount: 1.0,
            size: 25,
            pos,
            displacement: fastrand::f32() - 0.5,
        };
        if let Some(index) = self.items.iter().position(|item| item.pos >= pos) {
            self.items.insert(index, item);
        } else {
            self.items.push(item);
        }
    }

    pub fn drain_items_after_pos(&mut self, min_pos: i32) -> Vec<Thing> {
        if let Some((index, _)) = self
            .items
            .iter()
            .enumerate()
            .find(|(_, item)| item.pos + item.size / 2 >= min_pos)
        {
            self.items.drain(index..).rev().map(|i| i.thing).collect()
        } else {
            Vec::new()
        }
    }
}

pub struct ConveyorPlugin;

impl Plugin for ConveyorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets)
            .add_system_to_stage(CoreStage::PreUpdate, convey_items)
            .add_system_to_stage(CoreStage::PreUpdate, spawn_chains)
            .add_system(display_items)
            .add_system(debug_spawn_chains)
            .add_system(manage_dynamic_ghosts);
    }
}

#[derive(SystemParam)]
pub struct ConveyorSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    ghosts: Query<'w, 's, &'static mut DynamicGhost>,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct BeltDef(pub Vec3, pub Vec3, pub Vec3);

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

fn debug_belt_defs(debug: &mut DebugLines, defs: &[BeltDef], duration: f32) {
    for (i, BeltDef(a, m, b)) in defs.iter().copied().enumerate() {
        let color = if i % 2 == 0 { Color::WHITE } else { Color::RED };
        debug.line_colored(a, m, duration, color);
        debug.line_colored(m, b, duration, color);
    }
}

#[derive(Debug)]
pub struct ChainDef {
    from: ChainLink,
    to: ChainLink,
    over: Vec<Vec3>,
}

#[derive(Debug)]
pub enum ChainLink {
    Entity(Entity),
    Pos(Vec3),
}

impl ChainLink {
    fn entity(&self) -> Option<Entity> {
        match self {
            ChainLink::Entity(e) => Some(*e),
            ChainLink::Pos(_) => None,
        }
    }
}

fn debug_spawn_chains(input: Res<Input<KeyCode>>, mut chain_defs: Query<&mut ChainDef>) {
    if !input.just_pressed(KeyCode::Space) {
        return;
    }

    for mut def in chain_defs.iter_mut() {
        for pt in def.over.iter_mut() {
            let r = || 0.5 - fastrand::f32();
            *pt += Vec3::new(r(), 0.0, r());
        }
    }
}

pub struct DynamicGhost {
    from: Vec3,
    to: Entity,
    manifest: bool,
}

fn manage_dynamic_ghosts(
    mut frame: Local<usize>,
    mut chain_defs: Query<&mut ChainDef>,
    ghosts: Query<(Entity, &DynamicGhost), With<ChainDef>>,
    belts: Query<(Entity, &Transform, &ConveyorBelt)>,
    transforms: Query<&Transform>,
    mut cmds: Commands,
) {
    *frame += 1;
    if *frame < 2 {
        return;
    }
    *frame = 0;

    for (ghost_entity, ghost) in ghosts.iter() {
        let mut def = chain_defs.get_mut(ghost_entity).unwrap();

        if let Ok((e, _)) = snap_to_belt(&belts, ghost.from) {
            def.from = ChainLink::Entity(e);
        } else {
            def.from = ChainLink::Pos(ghost.from);
        }

        if let Ok(to) = transforms.get(ghost.to) {
            if let Ok((e, _)) = snap_to_belt(&belts, to.translation) {
                def.to = ChainLink::Entity(e);
            } else {
                def.to = ChainLink::Pos(to.translation);
            }
        } else {
            eprintln!("Try to conveyor ghost to an entity without Transform.");
        }

        if ghost.manifest {
            def.set_changed();
            cmds.entity(ghost_entity).remove::<DynamicGhost>();
        }
    }

    fn snap_to_belt(
        belts: &Query<(Entity, &Transform, &ConveyorBelt)>,
        from: Vec3,
    ) -> Result<(Entity, BeltDef), Vec3> {
        belts
            .iter()
            .map(|(e, _, belt)| (e, belt.def, belt.def.2.distance_squared(from)))
            .filter(|(_, _, d)| *d < 1.0)
            .min_by(|(_, _, a), (_, _, b)| {
                if a < b {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })
            .map(|(e, def, _)| Ok((e, def)))
            .unwrap_or(Err(from))
    }
}

fn spawn_chains(
    mut belts: Query<&mut ConveyorBelt>,
    transforms: Query<&Transform>,
    chain_defs: Query<
        (
            Entity,
            &ChainDef,
            Option<&ConveyorChain>,
            Option<&DynamicGhost>,
        ),
        Changed<ChainDef>,
    >,
    mut cmds: Commands,
    mut debug: ResMut<DebugLines>,
    assets: Res<ConveyorAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    debug_config: Res<DebugConfig>,
) {
    for (chain_def_entity, chain_def, chain, ghost) in chain_defs.iter() {
        let (begin_mid, begin) = match chain_def.from {
            ChainLink::Entity(e) => {
                if let Ok(belt) = belts.get_mut(e) {
                    let BeltDef(_, mid, begin) = belt.def;
                    (2.0 * begin - mid, Some(begin))
                } else if let Some(transform) =
                    chain_def.from.entity().and_then(|e| transforms.get(e).ok())
                {
                    (transform.translation, None)
                } else {
                    eprintln!("spawn_chains from entity does not match");
                    continue;
                }
            }
            ChainLink::Pos(mid) => (mid, None),
        };

        let (end_mid, end) = match chain_def.to {
            ChainLink::Entity(e) => {
                if let Ok(belt) = belts.get_mut(e) {
                    let BeltDef(end, mid, _) = belt.def;
                    (2.0 * end - mid, Some(end))
                } else if let Some(transform) =
                    chain_def.to.entity().and_then(|e| transforms.get(e).ok())
                {
                    (transform.translation, None)
                } else {
                    eprintln!("spawn_chains to entity does not match");
                    continue;
                }
            }
            ChainLink::Pos(mid) => (mid, None),
        };

        // begin, end, over -> control_defs

        let mut control_defs = Vec::<BeltDef>::new();

        if begin_mid.distance_squared(end_mid) <= 1.0 {
            // only space for one belt
            let mid = 0.5 * (begin_mid + end_mid);
            control_defs.push(match (begin, end) {
                (Some(begin), Some(end)) => BeltDef(begin, mid, end),
                (Some(begin), None) => {
                    BeltDef(begin, begin_mid, 0.5 * (end_mid - begin_mid).normalize())
                }
                (None, Some(end)) => BeltDef(0.5 * (begin_mid - end_mid).normalize(), end_mid, end),
                (None, None) => BeltDef(
                    0.5 * (begin_mid - mid).normalize(),
                    mid,
                    0.5 * (end_mid - mid).normalize(),
                ),
            });
        } else if chain_def.over.is_empty() {
            // no intermediate points
            let half_step = 0.5 * (end_mid - begin_mid).normalize();
            let begin = begin.unwrap_or_else(|| begin_mid - half_step);
            let end = end.unwrap_or_else(|| end_mid + half_step);
            control_defs.push(BeltDef(begin, begin_mid, begin_mid + half_step));
            control_defs.push(BeltDef(end_mid - half_step, end_mid, end));
        } else {
            let first_over = chain_def.over.first().unwrap().clone();
            let last_over = chain_def.over.last().unwrap().clone();

            let step_forward = 0.5 * (first_over - begin_mid).normalize();
            let begin_begin = begin.unwrap_or_else(|| begin_mid - step_forward);
            let begin_end = begin_mid + step_forward;

            let step_back = 0.5 * (last_over - end_mid).normalize();
            let end_end = end.unwrap_or_else(|| end_mid - step_back);
            let end_begin = end_mid + step_back;

            let begin_def = BeltDef(begin_begin, begin_mid, begin_end);
            let end_def = BeltDef(end_begin, end_mid, end_end);
            control_defs.push(begin_def);

            let len = chain_def.over.len();
            let mut mid_before = begin_mid;
            for index in 0..len {
                let mid = chain_def.over[index];
                let next_mid = if index == len - 1 {
                    end_mid
                } else {
                    chain_def.over[index + 1]
                };

                control_defs.push(BeltDef(
                    mid + 0.5 * (mid_before - mid).normalize(),
                    mid,
                    mid + 0.5 * (next_mid - mid).normalize(),
                ));
                mid_before = mid;
            }

            control_defs.push(end_def);
        }

        // control_defs -> belt_defs

        let mut belt_defs = Vec::<BeltDef>::new();

        if control_defs.len() == 1 {
            belt_defs = control_defs;
        } else {
            belt_defs.push(control_defs.first().unwrap().clone());

            for window in control_defs.windows(2) {
                let BeltDef(_, begin_mid, begin_end) = window[0].clone();
                let BeltDef(end_begin, end_mid, _) = window[1].clone();

                let way = end_begin - begin_end;
                let distance = way.length();
                let one_step = (end_mid - begin_mid).normalize();

                if distance < 0.0001 {
                    // both control points are next to each other
                    continue;
                }

                let mut begin = begin_end;
                let mut mid_before = begin_mid;
                let mut remaining_distance = distance;
                while remaining_distance >= 1.4 {
                    // create belt length = 1.0
                    let end = begin + one_step;
                    let mid = mid_before + one_step;
                    belt_defs.push(BeltDef(begin, mid, end));
                    remaining_distance -= 1.0;
                    begin = end;
                    mid_before = mid;
                }

                if remaining_distance >= 1.0 {
                    // create two belts length = remaining_distance / 2
                    let short_step = one_step * remaining_distance * 0.5;
                    let end = begin + short_step;
                    let mid = mid_before + short_step;
                    belt_defs.push(BeltDef(begin, mid, end));
                    belt_defs.push(BeltDef(
                        begin + short_step,
                        mid + short_step,
                        end + short_step,
                    ));
                } else {
                    // create belt length = remaining_distance
                    let short_step = one_step * remaining_distance;
                    let end = begin + short_step;
                    let mid = mid_before + short_step;
                    belt_defs.push(BeltDef(begin, mid, end));
                }

                belt_defs.push(window[1]);
            }
        }

        // belt_defs -> spawn

        let belt_entities: Vec<_> = belt_defs.iter().map(|_| cmds.spawn().id()).collect();

        if let Some(chain) = chain {
            for belt in chain.belts.iter() {
                cmds.entity(*belt).despawn_recursive();
            }
        }

        cmds.entity(chain_def_entity)
            .insert(ConveyorChain {
                from: chain_def.from.entity(),
                to: chain_def.to.entity(),
                belts: belt_entities.clone(),
            })
            .insert(Transform::identity())
            .insert(GlobalTransform::identity())
            .push_children(&belt_entities);

        if let Some(mut belt) = chain_def.from.entity().and_then(|e| belts.get_mut(e).ok()) {
            belt.output = belt_entities.first().cloned();
        }

        if debug_config.spawn_chains_belt_def_duration > 0.0 {
            debug_belt_defs(
                &mut debug,
                &belt_defs,
                debug_config.spawn_chains_belt_def_duration,
            );
        }

        let mut output = chain_def.to.entity();
        let mut flip = false;
        for (entity, def) in belt_entities.into_iter().zip(belt_defs.into_iter()).rev() {
            flip = !flip;

            let material = match ghost.is_some() {
                true => assets.ghost_material.clone(),
                false => assets.material.clone(),
            };

            let model = cmds
                .spawn_bundle(PbrBundle {
                    material,
                    mesh: meshes.add(curve(def.0, def.1, def.2, if flip { 0.8 } else { 0.7 })),
                    transform: Transform::from_xyz(0.0, 0.1, 0.0),
                    ..Default::default()
                })
                .insert(NotGround)
                .id();

            let transform = Transform::from_xyz(def.1.x, 0.05, def.1.z);

            if ghost.is_some() {
                cmds.entity(entity)
                    .push_children(&[model])
                    .insert_bundle((transform, GlobalTransform::identity()));
            } else {
                cmds.entity(entity).push_children(&[model]).insert_bundle((
                    ConveyorBelt {
                        output,
                        ..ConveyorBelt::new((def.2.distance(def.0) * 100.0).floor() as i32, def)
                    },
                    transform,
                    GlobalTransform::identity(),
                    Destructable,
                    FocusObject::new(),
                ));
            }

            output = Some(entity);
        }
    }
}

impl<'w, 's> ConveyorSpawn<'w, 's> {
    pub fn spawn_chain(&mut self, from: ChainLink, to: ChainLink) {
        self.cmds.spawn_bundle((ChainDef {
            from,
            to,
            over: Vec::new(),
        },));
    }

    pub fn spawn_chain_over(&mut self, from: ChainLink, to: ChainLink, over: &[Vec3]) {
        self.cmds.spawn_bundle((ChainDef {
            from,
            to,
            over: over.iter().copied().collect(),
        },));
    }

    pub(crate) fn spawn_dynamic_ghost(&mut self, from: Vec3, to: Entity) -> Entity {
        self.cmds
            .spawn_bundle((
                ChainDef {
                    from: ChainLink::Pos(from),
                    to: ChainLink::Entity(to),
                    over: Vec::new(),
                },
                DynamicGhost {
                    from,
                    to,
                    manifest: false,
                },
            ))
            .id()
    }

    pub(crate) fn manifest_ghost(&mut self, ghost: Entity) {
        for mut ghost in self.ghosts.get_mut(ghost) {
            ghost.manifest = true;
        }
    }
}

#[derive(Clone)]
pub struct ConveyorAssets {
    pub transform: Transform,
    pub material: Handle<StandardMaterial>,
    pub ghost_material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
    pub item_mesh: Handle<Mesh>,
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
        item_mesh: meshes.add(disk(0.5, 12)),
    });
}

struct ConveyorChain {
    from: Option<Entity>,
    to: Option<Entity>,
    belts: Vec<Entity>,
}

impl ConveyorBelt {
    fn left(&self) -> i32 {
        self.items.first().map(|i| i.left()).unwrap_or(self.length)
    }

    pub fn free_pos(&self, size: i32) -> Option<i32> {
        if self.items.is_empty() {
            return Some(size / 2);
        }

        let mut left = 0;
        for item in self.items.iter() {
            let space = item.left() - left;

            if space >= size {
                return Some(left + size / 2);
            } else {
                left = item.right();
            }
        }
        None
    }

    pub fn has_space(&self, size: i32) -> bool {
        self.free_pos(size).is_some()
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
        for belt_entity in chain
            .to
            .iter()
            .chain(chain.belts.iter().rev())
            .chain(chain.from.iter())
            .copied()
        {
            let belts = query_belts.q1();
            let belt = if let Ok(belt) = belts.get(belt_entity) {
                belt
            } else {
                continue;
            };

            if belt.items.len() > 0 {
                let next_belt = belt.output.and_then(|e| belts.get(e).ok());

                if let Some(next_belt) = next_belt {
                    let next_belt_left = next_belt.left();
                    let mut belts = query_belts.q0();
                    let mut belt = belts.get_mut(belt_entity).unwrap();
                    let mut transfer_index = None;
                    let speed = 5;
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
                    let speed = 5;
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

struct ItemModel;

fn display_items(
    belts: Query<(&ConveyorBelt, &Transform)>,
    models: Query<Entity, With<ItemModel>>,
    mut cmds: Commands,
    mut visible: Query<&mut Visible, With<ItemModel>>,
    assets: Res<ConveyorAssets>,
    thing_materials: Res<ThingMaterials>,
) {
    let mut models: Vec<Entity> = models.iter().collect();

    for (belt, _transform) in belts.iter() {
        let length_f32 = belt.length as f32;
        let BeltDef(a, m, b) = belt.def;
        let from = (a - m).to_point();
        let to = (b - m).to_point();
        let ctrl1 = from * 0.5;
        let ctrl2 = to * 0.5;
        let segment = CubicBezierSegment {
            from,
            ctrl1,
            ctrl2,
            to,
        };

        for item in belt.items.iter() {
            let item_size = item.size as f32 / 100.0;
            let linear_item_pos = item.pos as f32 / length_f32;
            let item_pos = m + segment.sample(linear_item_pos).to_vec3();
            let tangent = segment.derivative(linear_item_pos).normalize();
            let normal = Vec3::new(tangent.y, 0.0, -tangent.x);

            let transform = Transform {
                translation: item_pos + 0.25 * Vec3::Y + 0.2 * item.displacement * normal,
                rotation: Quat::from_rotation_y(tangent.y.atan2(tangent.x)),
                scale: item_size * Vec3::ONE,
            };
            let material = thing_materials.get(item.thing);

            if let Some(entity) = models.pop() {
                cmds.entity(entity).insert_bundle((
                    transform,
                    material,
                    Visible {
                        is_visible: true,
                        is_transparent: false,
                    },
                ));
            } else {
                cmds.spawn()
                    .insert_bundle(PbrBundle {
                        transform,
                        material,
                        mesh: assets.item_mesh.clone(),
                        ..Default::default()
                    })
                    .insert(ItemModel);
            }
        }
    }

    for model in models {
        for mut visible in visible.get_mut(model) {
            visible.is_visible = false;
        }
    }
}

#[allow(unused)]
fn debug_items(belts: Query<(&ConveyorBelt, &Transform)>, mut debug: ResMut<DebugLines>) {
    for (belt, _transform) in belts.iter() {
        let length_f32 = belt.length as f32;
        let BeltDef(a, m, b) = belt.def;
        let from = (a - m).to_point();
        let to = (b - m).to_point();
        let ctrl1 = from * 0.5;
        let ctrl2 = to * 0.5;
        let segment = CubicBezierSegment {
            from,
            ctrl1,
            ctrl2,
            to,
        };

        for item in belt.items.iter() {
            let item_size = item.size as f32 / 100.0;
            let linear_item_pos = item.pos as f32 / length_f32;
            let item_pos = m + segment.sample(linear_item_pos).to_vec3();
            let tangent = segment.derivative(linear_item_pos).normalize();
            let extent = tangent * item_size * 0.5;
            let side_extent = Vec3::new(extent.x, 0.0, extent.y);
            let length_extent = Vec3::new(extent.y, 0.0, -extent.x);
            let start = item_pos - length_extent;
            let end = item_pos + length_extent;

            let extent = side_extent;
            debug.line_colored(start - extent, start + extent, 0.0, Color::YELLOW);
            debug.line_colored(end - extent, end + extent, 0.0, Color::YELLOW);
            debug.line_colored(start - extent, end - extent, 0.0, Color::YELLOW);
            debug.line_colored(start + extent, end + extent, 0.0, Color::YELLOW);
        }
    }
}
