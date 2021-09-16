use std::{cmp::Ordering, f32::consts::TAU};

use bevy::{ecs::system::SystemParam, math::vec3, prelude::*};
use bevy_mod_picking::PickableBundle;
use bevy_prototype_debug_lines::DebugLines;

use crate::{
    entities::StoreIntoPile,
    systems::{BrainPlugin, DebugConfig, Destructable, FunnyAnimation, Thing},
};

use super::{
    tree::{self, MarkCutTree},
    Boulder, ConveyorBelt,
};

#[derive(Clone, Default)]
pub struct Imp {
    pub idle_time: f32,
    pub idle_new_direction_time: f32,
    pub work_time: f32,
    pub load: Option<Thing>,
    pub load_amount: f32,
    pub walk_destination: WalkDestination,
    pub want_to_follow: Option<Entity>,
    pub idle_complete: bool,
    pub boulder: Option<Entity>,
    pub conveyor: Option<Entity>,
    pub tree: Option<Entity>,
}

#[derive(Clone, Copy, Debug)]
pub enum WalkDestination {
    None,
    Vec3(Vec3),
    Entity(Entity),
}

impl Default for WalkDestination {
    fn default() -> Self {
        Self::None
    }
}

impl From<Option<Entity>> for WalkDestination {
    fn from(o: Option<Entity>) -> Self {
        o.map(|e| WalkDestination::Entity(e)).unwrap_or_default()
    }
}

impl Imp {
    pub fn new() -> Self {
        Self {
            idle_time: 1.0,
            idle_new_direction_time: 1.0,
            walk_destination: WalkDestination::None,
            ..Self::default()
        }
    }

    pub fn maybe_follow(&mut self, entity: Entity) {
        self.want_to_follow = Some(entity);
    }

    pub fn remove_thing(&mut self, thing: Thing) {
        if self.load == Some(thing) && self.load_amount >= 1.0 {
            self.load_amount -= 1.0;
        }
    }
}

#[derive(Clone)]
pub struct ImpAssets {
    pub transform: Transform,
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
}

pub struct ImpPlugin;

impl Plugin for ImpPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets)
            .add_system_to_stage(CoreStage::PostUpdate, update_walk)
            .add_system_to_stage(CoreStage::PostUpdate, debug_lines)
            .add_plugin(ImpBrainPlugin);
    }
}

#[derive(SystemParam)]
pub struct ImpSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, ImpAssets>,
}

pub struct ImpModel;

impl<'w, 's> ImpSpawn<'w, 's> {
    pub fn spawn(&mut self, imp: Imp, transform: Transform) {
        let model = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: self.assets.transform.clone(),
                material: self.assets.material.clone(),
                mesh: self.assets.mesh.clone(),
                ..Default::default()
            })
            .insert(ImpModel)
            .insert_bundle(PickableBundle::default())
            .id();

        self.cmds
            .spawn_bundle((
                imp,
                transform,
                GlobalTransform::identity(),
                Destructable,
                brain(),
            ))
            .push_children(&[model]);
    }
}

fn load_assets(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.insert_resource(ImpAssets {
        transform: Transform::from_xyz(0.0, 0.3, 0.0),
        material: materials.add(material(Color::SALMON)),
        mesh: meshes.add(shape::Box::new(0.4, 0.6, 0.4).into()),
    });

    fn material(color: Color) -> StandardMaterial {
        StandardMaterial {
            base_color: color,
            ..Default::default()
        }
    }
}

fn update_walk(
    time: Res<Time>,
    mut imps: Query<(&Imp, &mut Transform)>,
    destination: Query<&Transform, Without<Imp>>,
) {
    let dt = time.delta_seconds();

    for (imp, mut transform) in imps.iter_mut() {
        let destination = match imp.walk_destination {
            WalkDestination::None => continue,
            WalkDestination::Vec3(vec) => vec,
            WalkDestination::Entity(entity) => {
                if let Ok(t) = destination.get(entity) {
                    t.translation
                } else {
                    continue;
                }
            }
        };

        let diff = destination - transform.translation;
        let len2 = diff.length_squared();
        let vec = if len2 < 1.0 { diff } else { diff / len2.sqrt() };
        let speed = 3.0;
        let step = vec * speed * dt;
        transform.translation += step;
    }
}

use big_brain::prelude::*;

struct ImpBrainPlugin;
impl Plugin for ImpBrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::First, want_to_dig)
            .add_system_to_stage(CoreStage::First, want_to_store)
            .add_system_to_stage(CoreStage::First, want_to_drop)
            .add_system_to_stage(CoreStage::First, want_to_cut_tree)
            .add_system(do_store)
            .add_system(do_cut_tree)
            .add_system(do_dig)
            .add_system(do_drop)
            .add_system(do_meander)
            .add_plugin(BrainPlugin);
    }
}

fn brain() -> ThinkerBuilder {
    Thinker::build()
        .picker(FirstToScore { threshold: 0.8 })
        .when(WantToStoreBuilder, DoStoreBuilder)
        .when(WantToCutTreeBuilder, DoCutTreeBuilder)
        .when(WantToDigBuilder, DoDigBuilder)
        .when(WantToDropBuilder, DoDropBuilder)
        .otherwise(DoMeanderBuilder)
}

struct WantToStore;
struct WantToCutTree;
struct WantToDig;
struct WantToDrop;
struct DoStore;
struct DoCutTree;
struct DoDig;
struct DoDrop;
struct DoMeander;

#[derive(Debug, Clone)]
struct WantToStoreBuilder;
impl ScorerBuilder for WantToStoreBuilder {
    fn build(&self, cmd: &mut Commands, scorer: Entity, _actor: Entity) {
        cmd.entity(scorer).insert(WantToStore);
    }
}

#[derive(Debug, Clone)]
struct WantToCutTreeBuilder;
impl ScorerBuilder for WantToCutTreeBuilder {
    fn build(&self, cmd: &mut Commands, scorer: Entity, _actor: Entity) {
        cmd.entity(scorer).insert(WantToCutTree);
    }
}

#[derive(Debug, Clone)]
struct WantToDigBuilder;
impl ScorerBuilder for WantToDigBuilder {
    fn build(&self, cmd: &mut Commands, scorer: Entity, _actor: Entity) {
        cmd.entity(scorer).insert(WantToDig);
    }
}

#[derive(Debug, Clone)]
struct WantToDropBuilder;
impl ScorerBuilder for WantToDropBuilder {
    fn build(&self, cmd: &mut Commands, scorer: Entity, _actor: Entity) {
        cmd.entity(scorer).insert(WantToDrop);
    }
}

#[derive(Debug, Clone)]
struct DoStoreBuilder;
impl ActionBuilder for DoStoreBuilder {
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action)
            .insert(DoStore)
            .insert(ActionState::Requested);
    }
}

#[derive(Debug, Clone)]
struct DoCutTreeBuilder;
impl ActionBuilder for DoCutTreeBuilder {
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action)
            .insert(DoCutTree)
            .insert(ActionState::Requested);
    }
}

#[derive(Debug, Clone)]
struct DoDigBuilder;
impl ActionBuilder for DoDigBuilder {
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action)
            .insert(DoDig)
            .insert(ActionState::Requested);
    }
}

#[derive(Debug, Clone)]
struct DoDropBuilder;
impl ActionBuilder for DoDropBuilder {
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action)
            .insert(DoDrop)
            .insert(ActionState::Requested);
    }
}

#[derive(Debug, Clone)]
struct DoMeanderBuilder;
impl ActionBuilder for DoMeanderBuilder {
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action)
            .insert(DoMeander)
            .insert(ActionState::Requested);
    }
}

////////

fn want_to_store(
    imps: Query<&Imp>,
    mut query: Query<(&Actor, &mut Score), With<WantToStore>>,
    conveyors: Query<&ConveyorBelt>,
) {
    let things: Vec<Thing> = conveyors
        .iter()
        .filter_map(|it| it.marked_for_thing)
        .collect();

    for (Actor(actor), mut score) in query.iter_mut() {
        for imp in imps.get(*actor) {
            if imp.conveyor.is_some() {
                score.set(1.0)
            } else if imp.load_amount >= 1.0 && things.contains(&imp.load.unwrap()) {
                score.set(1.0);
            } else {
                score.set(0.0);
            }
        }
    }
}

fn do_store(
    mut imps: Query<(&mut Imp, &Transform)>,
    mut belts: Query<(Entity, &mut ConveyorBelt, &Transform)>,
    mut query: Query<(&Actor, &mut ActionState), With<DoStore>>,
    mut cmds: Commands,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let cmds = &mut cmds;

    for (Actor(actor), mut state) in query.iter_mut() {
        let imp_entity = *actor;

        if let Ok((mut imp, transform)) = imps.get_mut(imp_entity) {
            let imp = &mut imp;
            let pos = transform.translation;

            fn init(imp: &mut Mut<Imp>, imp_entity: Entity, cmds: &mut Commands) {
                imp.conveyor = None;
                imp.work_time = 0.0;
                imp.walk_destination = WalkDestination::None;
                cmds.entity(imp_entity).remove::<FunnyAnimation>();
            }

            fn find(
                imp: &mut Mut<Imp>,
                pos: Vec3,
                belts: &mut Query<(Entity, &mut ConveyorBelt, &Transform)>,
            ) {
                imp.conveyor = belts
                    .iter_mut()
                    .filter(|(_, it, _)| it.marked_for_thing == imp.load)
                    .map(|(e, c, t)| (t.translation.distance_squared(pos), e, c, t))
                    .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Less))
                    .map(|p| p.1);

                imp.work_time = 0.0;
                imp.walk_destination = imp.conveyor.into();
            }

            fn execute(
                imp: &mut Mut<Imp>,
                belts: &mut Query<(Entity, &mut ConveyorBelt, &Transform)>,
                state: &mut Mut<ActionState>,
                pos: Vec3,
                imp_entity: Entity,
                cmds: &mut Commands,
                dt: f32,
            ) {
                let state = &mut **state;
                if let Some((_, mut belt, transform)) =
                    imp.conveyor.and_then(|e| belts.get_mut(e).ok())
                {
                    if belt.marked_for_thing != imp.load {
                        *state = ActionState::Failure;
                    } else if imp.load_amount == 0.0 {
                        imp.load = None;

                        *state = ActionState::Failure;
                    } else if pos.distance_squared(transform.translation) < 1.0 {
                        imp.walk_destination = WalkDestination::None;

                        if imp.work_time == 0.0 {
                            cmds.entity(imp_entity)
                                .insert(FunnyAnimation { offset: 0.0 });
                        }

                        imp.work_time += dt;
                        let thing = imp.load.unwrap();

                        if imp.work_time >= 1.0 && belt.has_space(25) {
                            imp.remove_thing(thing);
                            belt.put_thing(thing);

                            init(imp, imp_entity, cmds);
                            *state = ActionState::Success;
                        } else {
                            // not finished yet, but working
                        }
                    } else {
                        // not there yet, but moving
                    }
                } else {
                    *state = ActionState::Failure;
                }
            }

            match *state {
                ActionState::Init => {
                    init(imp, imp_entity, cmds);
                }
                ActionState::Requested => {
                    find(imp, pos, &mut belts);
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    //
                    execute(imp, &mut belts, &mut state, pos, imp_entity, cmds, dt)
                }
                ActionState::Cancelled => {
                    *state = ActionState::Failure;
                }
                ActionState::Success | ActionState::Failure => {}
            }
        }
    }
}

////////

fn want_to_drop(imps: Query<&Imp>, mut query: Query<(&Actor, &mut Score), With<WantToDrop>>) {
    for (Actor(actor), mut score) in query.iter_mut() {
        if let Ok(imp) = imps.get(*actor) {
            if imp.load_amount >= 1.0 {
                score.set(0.9);
            } else {
                score.set(0.0);
            }
        }
    }
}

fn do_drop(
    mut imps: Query<(&mut Imp, &Transform)>,
    mut query: Query<(&Actor, &mut ActionState), With<DoDrop>>,
    mut cmds: Commands,
) {
    for (Actor(actor), mut state) in query.iter_mut() {
        if let Ok((mut imp, transform)) = imps.get_mut(*actor) {
            let pos = transform.translation;

            match *state {
                ActionState::Requested => {
                    imp.walk_destination = WalkDestination::Vec3(pos + 1.5 * random_vec());
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    if let WalkDestination::Vec3(dest) = imp.walk_destination {
                        if pos.distance_squared(dest) < 0.1 {
                            let drop_amount = imp.load_amount.max(1.0);
                            imp.load_amount = imp.load_amount - drop_amount;

                            cmds.spawn_bundle((
                                StoreIntoPile {
                                    load: imp.load.unwrap(),
                                    amount: drop_amount,
                                    pile: None,
                                },
                                transform.clone(),
                            ));

                            if imp.load_amount == 0.0 {
                                imp.load = None;
                            }

                            *state = ActionState::Success;
                        }
                    }
                }
                ActionState::Cancelled => {
                    if imp.load_amount < 1.0 {
                        *state = ActionState::Success;
                    } else {
                        *state = ActionState::Failure;
                    }
                }
                ActionState::Success | ActionState::Failure => {
                    imp.walk_destination = WalkDestination::None;
                }
                _ => {}
            }
        }
    }
}

fn want_to_dig(
    imps: Query<&Imp>,
    boulders: Query<&Boulder>,
    mut query: Query<(&Actor, &mut Score), With<WantToDig>>,
) {
    for (Actor(actor), mut score) in query.iter_mut() {
        if let Ok(imp) = imps.get(*actor) {
            if imp.load_amount < 1.0 {
                if let Some(thing) = imp.load {
                    if boulders
                        .iter()
                        .filter(|boulder| {
                            boulder.marked_for_digging && thing == boulder.material.into()
                        })
                        .next()
                        .is_some()
                    {
                        score.set(1.0);
                    } else {
                        score.set(0.0);
                    }
                } else if boulders
                    .iter()
                    .filter(|boulder| boulder.marked_for_digging)
                    .next()
                    .is_some()
                {
                    score.set(1.0);
                } else {
                    score.set(0.0);
                }
            } else {
                score.set(0.0);
            }
        }
    }
}

fn do_dig(
    mut imps: Query<(&mut Imp, &Transform)>,
    boulders: Query<(Entity, &Boulder, &Transform)>,
    mut query: Query<(&Actor, &mut ActionState), With<DoDig>>,
    time: Res<Time>,
    mut cmds: Commands,
) {
    let dt = time.delta_seconds();

    for (Actor(actor), mut state) in query.iter_mut() {
        if let Ok((mut imp, transform)) = imps.get_mut(*actor) {
            let pos = transform.translation;

            match *state {
                ActionState::Init => {}
                ActionState::Requested => {
                    imp.boulder = boulders
                        .iter()
                        .filter(|(_, boulder, _)| boulder.marked_for_digging)
                        .filter(|(_, boulder, _)| {
                            imp.load.is_none() || imp.load == Some(boulder.material.into())
                        })
                        .map(|(entity, _, transform)| {
                            (entity, pos.distance_squared(transform.translation))
                        })
                        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Less))
                        .map(|(e, _)| e);

                    imp.walk_destination = imp.boulder.into();

                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    if let Some(entity) = imp.boulder {
                        if let Ok((_, boulder, transform)) = boulders.get(entity) {
                            let material = Some(boulder.material.into());

                            if !boulder.marked_for_digging {
                                *state = ActionState::Failure;
                            } else if imp.load.is_some() && imp.load != material {
                                *state = ActionState::Failure;
                            } else if imp.load_amount < 1.0 {
                                if imp.load.is_none() {
                                    imp.load = material;
                                }

                                if pos.distance_squared(transform.translation) < 1.0 {
                                    imp.walk_destination = WalkDestination::None;
                                    imp.load_amount = (imp.load_amount + dt).min(1.0);
                                    cmds.entity(*actor).insert(FunnyAnimation { offset: 0.0 });
                                }
                            } else {
                                *state = ActionState::Success;
                            }
                        } else {
                            *state = ActionState::Failure;
                        }
                    } else {
                        *state = ActionState::Failure;
                    }
                }
                ActionState::Cancelled => {
                    if imp.load_amount < 1.0 {
                        *state = ActionState::Failure;
                    } else {
                        *state = ActionState::Success;
                    }
                }
                ActionState::Success | ActionState::Failure => {
                    imp.boulder = None;
                    imp.walk_destination = WalkDestination::None;
                    cmds.entity(*actor).remove::<FunnyAnimation>();
                }
            }
        }
    }
}

fn do_meander(
    mut imps: Query<(&mut Imp, &Transform)>,
    mut query: Query<(&Actor, &mut ActionState), With<DoMeander>>,
) {
    for (Actor(actor), mut state) in query.iter_mut() {
        if let Ok((mut imp, transform)) = imps.get_mut(*actor) {
            let pos = transform.translation;

            match *state {
                ActionState::Init => {}
                ActionState::Requested => {
                    imp.walk_destination = WalkDestination::Vec3(pos + 2.0 * random_vec());
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    if let WalkDestination::Vec3(dest) = imp.walk_destination {
                        if pos.distance_squared(dest) < 0.1 {
                            *state = ActionState::Success;
                        }
                    }
                }
                ActionState::Cancelled => {
                    *state = ActionState::Success;
                }
                ActionState::Success => {
                    imp.walk_destination = WalkDestination::None;
                }
                ActionState::Failure => {}
            }
        }
    }
}

///////

fn random_vec() -> Vec3 {
    let a = TAU * fastrand::f32();
    vec3(a.cos(), 0.0, a.sin())
}

///////

fn debug_lines(
    config: Res<DebugConfig>,
    mut debug: ResMut<DebugLines>,
    imps: Query<(&Imp, &Transform)>,
    transforms: Query<&Transform>,
) {
    if !config.imp_walk_destination {
        return;
    }

    for (imp, transform) in imps.iter() {
        let pos = transform.translation;

        let dest = match imp.walk_destination {
            WalkDestination::None => None,
            WalkDestination::Vec3(dest) => Some(dest),
            WalkDestination::Entity(entity) => transforms.get(entity).ok().map(|t| t.translation),
        };

        if let Some(dest) = dest {
            debug.line_colored(pos, dest, 0.0, Color::BLUE);
        }
    }
}

fn want_to_cut_tree(
    imps: Query<&Imp>,
    trees: Query<&tree::Component, With<MarkCutTree>>,
    mut query: Query<(&Actor, &mut Score), With<WantToCutTree>>,
) {
    let trees_count = trees.iter().count();

    for (Actor(actor), mut score) in query.iter_mut() {
        let imp = imps.get(*actor).unwrap();

        if trees_count > 0
            && (imp.load == None || imp.load == Some(Thing::Wood) && imp.load_amount < 1.0)
        {
            score.set(1.0);
        } else {
            score.set(0.0);
        }
    }
}

fn do_cut_tree(
    mut imps: Query<(&mut Imp, &Transform)>,
    mut trees: Query<(Entity, &mut tree::Component, &Transform), With<MarkCutTree>>,
    mut query: Query<(&Actor, &mut ActionState), With<DoCutTree>>,
    time: Res<Time>,
    mut cmds: Commands,
) {
    let dt = time.delta_seconds();

    for (Actor(actor), mut state) in query.iter_mut() {
        if let Ok((mut imp, transform)) = imps.get_mut(*actor) {
            let pos = transform.translation;

            if *state == ActionState::Requested {
                imp.tree = trees
                    .iter_mut()
                    .map(|(entity, _, transform)| {
                        (entity, pos.distance_squared(transform.translation))
                    })
                    .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Less))
                    .map(|(e, _)| e);

                imp.walk_destination = imp.tree.into();

                *state = ActionState::Executing;
            }

            if *state == ActionState::Cancelled {
                if imp.load_amount < 1.0 {
                    *state = ActionState::Failure;
                } else {
                    *state = ActionState::Success;
                }
            }

            if *state == ActionState::Executing {
                if let Some(entity) = imp.tree {
                    if let Ok((_, mut t, transform)) = trees.get_mut(entity) {
                        if imp.load_amount < 1.0 {
                            if pos.distance_squared(transform.translation) < 1.0 {
                                let mass = t.cut(dt);
                                imp.walk_destination = WalkDestination::None;
                                imp.load = Some(Thing::Wood);
                                imp.load_amount += mass;
                                cmds.entity(*actor).insert(FunnyAnimation { offset: 0.0 });
                            }
                        } else {
                            *state = ActionState::Success;
                        }
                    } else {
                        *state = ActionState::Failure;
                    }
                } else {
                    *state = ActionState::Failure;
                }
            }

            if *state == ActionState::Failure || *state == ActionState::Success {
                imp.tree = None;
                imp.walk_destination = WalkDestination::None;
                cmds.entity(*actor).remove::<FunnyAnimation>();
            }
        }
    }
}
