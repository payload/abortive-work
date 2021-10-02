use std::cmp::Ordering;

use bevy::{ecs::system::SystemParam, prelude::*, utils::HashMap};
use bevy_prototype_debug_lines::DebugLines;
use lyon::geom::CubicBezierSegment;

use crate::{
    extensions::{QueryExt, ToPoint, ToVec3},
    systems::{curve, DebugConfig, Destructable, FocusObject, Thing},
};

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
                },
            );
        }
    }

    pub fn drain_items_after_pos(&mut self, min_pos: i32) {
        if let Some((index, _)) = self
            .items
            .iter()
            .enumerate()
            .find(|(_, item)| item.pos + item.size / 2 >= min_pos)
        {
            self.items.drain(index..);
        }
    }
}

pub struct ConveyorPlugin;

impl Plugin for ConveyorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets)
            .add_system_to_stage(CoreStage::Update, update_ghostchains)
            .add_system_to_stage(CoreStage::PreUpdate, convey_items)
            .add_system_to_stage(CoreStage::PostUpdate, debug_items)
            .add_system(spawn_chains)
            .add_system(debug_spawn_chains);
    }
}

#[derive(SystemParam)]
pub struct ConveyorSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, ConveyorAssets>,
    belts: Query<'w, 's, (&'static Transform, &'static ConveyorBelt)>,
    debug: ResMut<'w, DebugLines>,
    meshes: ResMut<'w, Assets<Mesh>>,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct BeltDef(pub Vec3, pub Vec3, pub Vec3);

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

fn debug_belt_defs(debug: &mut DebugLines, defs: &[BeltDef], duration: f32) {
    for (i, BeltDef(a, m, b)) in defs.iter().copied().enumerate() {
        let color = if i % 2 == 0 { Color::WHITE } else { Color::RED };
        debug.line_colored(a, m, duration, color);
        debug.line_colored(m, b, duration, color);
    }
}

struct ChainDef {
    from: ChainLink,
    to: ChainLink,
    over: Vec<Vec3>,
}

pub enum ChainLink {
    Entity(Entity),
    Pos(Vec3),
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

fn spawn_chains(
    belts: Query<&ConveyorBelt>,
    chain_defs: Query<(Entity, &ChainDef, Option<&ConveyorChain>), Changed<ChainDef>>,
    mut cmds: Commands,
    mut debug: ResMut<DebugLines>,
    assets: Res<ConveyorAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    debug_config: Res<DebugConfig>,
) {
    for (chain_def_entity, def, chain) in chain_defs.iter() {
        let (begin_mid, begin) = match def.from {
            ChainLink::Entity(e) => {
                if let Ok(belt) = belts.get(e) {
                    let BeltDef(_, mid, begin) = belt.def;
                    (2.0 * begin - mid, Some(begin))
                } else {
                    continue;
                }
            }
            ChainLink::Pos(mid) => (mid, None),
        };

        let (end_mid, end) = match def.to {
            ChainLink::Entity(e) => {
                if let Ok(belt) = belts.get(e) {
                    let BeltDef(end, mid, _) = belt.def;
                    (2.0 * end - mid, Some(end))
                } else {
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
        } else if def.over.is_empty() {
            // no intermediate points
            let half_step = 0.5 * (end_mid - begin_mid).normalize();
            let begin = begin.unwrap_or_else(|| begin_mid - half_step);
            let end = end.unwrap_or_else(|| end_mid + half_step);
            control_defs.push(BeltDef(begin, begin_mid, begin_mid + half_step));
            control_defs.push(BeltDef(end_mid - half_step, end_mid, end));
        } else {
            let first_over = def.over.first().unwrap().clone();
            let last_over = def.over.last().unwrap().clone();

            let step_forward = 0.5 * (first_over - begin_mid).normalize();
            let begin_begin = begin.unwrap_or_else(|| begin_mid - step_forward);
            let begin_end = begin_mid + step_forward;

            let step_back = 0.5 * (last_over - end_mid).normalize();
            let end_end = end.unwrap_or_else(|| end_mid - step_back);
            let end_begin = end_mid + step_back;

            let begin_def = BeltDef(begin_begin, begin_mid, begin_end);
            let end_def = BeltDef(end_begin, end_mid, end_end);
            control_defs.push(begin_def);

            let len = def.over.len();
            let mut mid_before = begin_mid;
            for index in 0..len {
                let mid = def.over[index];
                let next_mid = if index == len - 1 {
                    end_mid
                } else {
                    def.over[index + 1]
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

        let defs: Vec<_> = belt_defs
            .into_iter()
            .map(|def| (cmds.spawn().id(), def))
            .collect();

        if let Some(chain) = chain {
            for belt in chain.belts.iter() {
                cmds.entity(*belt).despawn_recursive();
            }
        }

        cmds.entity(chain_def_entity).insert(ConveyorChain {
            belts: defs.iter().map(|p| p.0).collect(),
        });

        if debug_config.spawn_chains_belt_def_duration > 0.0 {
            debug_belt_defs(
                &mut debug,
                &defs.iter().map(|(_, def)| def).copied().collect::<Vec<_>>(),
                debug_config.spawn_chains_belt_def_duration,
            );
        }

        let mut output = if let ChainLink::Entity(e) = def.to {
            Some(e)
        } else {
            None
        };

        let mut flip = false;
        for (entity, def) in defs.into_iter().rev() {
            let model = cmds
                .spawn_bundle(PbrBundle {
                    material: assets.material.clone(),
                    mesh: meshes.add(curve(def.0, def.1, def.2, if flip { 0.8 } else { 0.7 })),
                    transform: Transform::from_xyz(0.0, 0.1, 0.0),
                    ..Default::default()
                })
                .insert(NotGround)
                .id();

            flip = !flip;

            cmds.entity(entity).push_children(&[model]).insert_bundle((
                ConveyorBelt {
                    output,
                    ..ConveyorBelt::new((def.2.distance(def.0) * 100.0).floor() as i32, def)
                },
                Transform {
                    rotation: Quat::IDENTITY,
                    translation: Vec3::new(def.1.x, 0.1, def.1.z),
                    scale: Vec3::ONE,
                },
                GlobalTransform::identity(),
                Destructable,
                FocusObject::new(),
            ));
            output = Some(entity);
        }
    }
}

pub fn create_chain_definition(points: &[Vec3], output: Option<BeltDef>) -> Vec<BeltDef> {
    assert!(points.len() >= 2);
    let mut from = points[0];
    let mut chain_of_mid_points = vec![from];

    for to in &points[1..] {
        chain_of_mid_points.extend(&get_mids(from, *to)[1..]);
        from = *chain_of_mid_points.last().unwrap();
    }

    let defs: Vec<_> = (0..chain_of_mid_points.len())
        .map(|index| {
            if index > 0 && index < chain_of_mid_points.len() - 1 {
                let before = chain_of_mid_points[index - 1];
                let mid = chain_of_mid_points[index];
                let after = chain_of_mid_points[index + 1];
                BeltDef(mid + 0.5 * (before - mid), mid, mid + 0.5 * (after - mid))
            } else if index == 0 {
                let mid = chain_of_mid_points[index];
                let after = chain_of_mid_points[index + 1];
                BeltDef(mid - 0.5 * (after - mid), mid, mid + 0.5 * (after - mid))
            } else {
                if let Some(belt_def) = output {
                    let before = chain_of_mid_points[index - 1];
                    let mid = chain_of_mid_points[index];
                    let end = belt_def.0;
                    BeltDef(mid + 0.5 * (before - mid), mid, end)
                } else {
                    let before = chain_of_mid_points[index - 1];
                    let mid = chain_of_mid_points[index];
                    BeltDef(mid + 0.5 * (before - mid), mid, mid - 0.5 * (before - mid))
                }
            }
        })
        .collect();
    defs
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

    pub fn create_chain_definition(
        &mut self,
        points: &[Vec3],
        output: Option<BeltDef>,
    ) -> Vec<(Option<Entity>, BeltDef)> {
        assert!(points.len() >= 2);
        let mut from = points[0];
        let mut chain_of_mid_points = vec![from];

        for to in &points[1..] {
            chain_of_mid_points.extend(&get_mids(from, *to)[1..]);
            from = *chain_of_mid_points.last().unwrap();
        }

        let defs: Vec<_> = (0..chain_of_mid_points.len())
            .map(|index| {
                if index > 0 && index < chain_of_mid_points.len() - 1 {
                    let before = chain_of_mid_points[index - 1];
                    let mid = chain_of_mid_points[index];
                    let after = chain_of_mid_points[index + 1];
                    BeltDef(mid + 0.5 * (before - mid), mid, mid + 0.5 * (after - mid))
                } else if index == 0 {
                    let mid = chain_of_mid_points[index];
                    let after = chain_of_mid_points[index + 1];
                    BeltDef(mid - 0.5 * (after - mid), mid, mid + 0.5 * (after - mid))
                } else {
                    if let Some(belt_def) = output {
                        let before = chain_of_mid_points[index - 1];
                        let mid = chain_of_mid_points[index];
                        let end = belt_def.0;
                        BeltDef(mid + 0.5 * (before - mid), mid, end)
                    } else {
                        let before = chain_of_mid_points[index - 1];
                        let mid = chain_of_mid_points[index];
                        BeltDef(mid + 0.5 * (before - mid), mid, mid - 0.5 * (before - mid))
                    }
                }
            })
            .map(|def| (None, def))
            .collect();
        defs
    }

    pub fn build_chain(&mut self, points: &[Vec3], output: Option<Entity>) -> (Entity, Entity) {
        assert!(points.len() >= 2);
        let mut from = points[0];
        let mut chain_of_mid_points = vec![from];

        for to in &points[1..] {
            chain_of_mid_points.extend(&get_mids(from, *to)[1..]);
            from = *chain_of_mid_points.last().unwrap();
        }

        let output_belt_def = self.belts.get_some(output).map(|(_, belt)| belt.def);
        let output_belt_def = output.and_then(|e| self.belts.get(e).map(|(_, belt)| belt.def).ok());

        //debug_vec3_strip(&mut self.debug, &chain);

        let defs: Vec<(Entity, BeltDef)> = (0..chain_of_mid_points.len())
            .map(|index| {
                if index > 0 && index < chain_of_mid_points.len() - 1 {
                    let before = chain_of_mid_points[index - 1];
                    let mid = chain_of_mid_points[index];
                    let after = chain_of_mid_points[index + 1];
                    BeltDef(mid + 0.5 * (before - mid), mid, mid + 0.5 * (after - mid))
                } else if index == 0 {
                    let mid = chain_of_mid_points[index];
                    let after = chain_of_mid_points[index + 1];
                    BeltDef(mid - 0.5 * (after - mid), mid, mid + 0.5 * (after - mid))
                } else {
                    if let Some(belt_def) = output_belt_def {
                        let before = chain_of_mid_points[index - 1];
                        let mid = chain_of_mid_points[index];
                        let end = belt_def.0;
                        BeltDef(mid + 0.5 * (before - mid), mid, end)
                    } else {
                        let before = chain_of_mid_points[index - 1];
                        let mid = chain_of_mid_points[index];
                        BeltDef(mid + 0.5 * (before - mid), mid, mid - 0.5 * (before - mid))
                    }
                }
            })
            .map(|def| (self.cmds.spawn().id(), def))
            .collect();

        let start_entity = defs.first().expect("defs first").0;
        let end_entity = defs.last().expect("defs last").0;

        self.cmds.spawn().insert(ConveyorChain {
            belts: defs.iter().map(|p| p.0).chain(output.into_iter()).collect(),
        });

        let mut output = output;
        for (entity, def) in defs.into_iter().rev() {
            let model = self.hq_model(self.assets.material.clone(), &def);
            self.cmds
                .entity(entity)
                .push_children(&[model])
                .insert_bundle((
                    ConveyorBelt {
                        output,
                        ..ConveyorBelt::new(100, def)
                    },
                    Transform {
                        rotation: Quat::IDENTITY,
                        translation: def.1,
                        scale: Vec3::ONE,
                    },
                    GlobalTransform::identity(),
                    Destructable,
                    FocusObject::new(),
                ));
            output = Some(entity);
        }

        (start_entity, end_entity)
    }

    pub fn spawn_line<'a>(&'a mut self, from: Vec3, to: Vec3) {
        eprintln!("warning, spawn_line, bad implementation");
        // let from = self.snap_to_belt(from);
        // let to = self.snap_to_belt(to);

        let mut line: Vec<_> = Self::iter_steps(from, to)
            .map(|(pos, angle)| {
                (
                    self.cmds.spawn().id(),
                    ConveyorBelt::new(100, BeltDef::default()),
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
                    FocusObject::new(),
                ))
                .push_children(&[model]);
        }
    }

    pub fn snap_to_belt(&self, from: Vec3) -> Result<BeltDef, Vec3> {
        self.belts
            .iter()
            .map(|(_, belt)| (belt.def, belt.def.2.distance_squared(from)))
            .filter(|(_, d)| *d < 1.0)
            .min_by(|(_, a), (_, b)| {
                if a < b {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })
            .map(|(def, _)| Ok(def))
            .unwrap_or(Err(from))
    }

    pub fn ghostline_from_point_to_entity(&mut self, from: Vec3, to: Entity) -> Entity {
        self.cmds
            .spawn_bundle((
                GhostchainFromHere { to },
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

    fn hq_model(&mut self, material: Handle<StandardMaterial>, def: &BeltDef) -> Entity {
        self.cmds
            .spawn_bundle(PbrBundle {
                material,
                mesh: self.meshes.add(curve(def.0, def.1, def.2, 0.8)),
                transform: Transform::from_xyz(0.0, 0.1, 0.0),
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

struct GhostchainFromHere {
    to: Entity,
}

struct GhostchainParent;

fn update_ghostchains(
    mut frame: Local<usize>,
    mut cmds: Commands,
    mut conveyor: ConveyorSpawn,
    chains: Query<(&Transform, &GhostchainFromHere)>,
    transforms: Query<&Transform>,
    parents: Query<Entity, With<GhostchainParent>>,
) {
    if *frame < 2 {
        *frame += 1;
        return;
    }

    *frame = 0;

    for parent in parents.iter() {
        cmds.entity(parent).despawn_recursive();
    }

    for (from_transform, from_here) in chains.iter() {
        if let Ok(to_transform) = transforms.get(from_here.to) {
            let parent = cmds
                .spawn_bundle((
                    GhostchainParent,
                    Transform::identity(),
                    GlobalTransform::identity(),
                ))
                .id();

            let from = conveyor.snap_to_belt(from_transform.translation);
            let to = conveyor.snap_to_belt(to_transform.translation);

            match (from, to) {
                (Err(from), Err(to)) => conveyor.spawn_ghostline(from, to, parent),
                _ => {}
            }
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
