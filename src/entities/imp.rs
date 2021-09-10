use std::{cmp::Ordering, f32::consts::TAU};

use bevy::{
    ecs::system::{EntityCommands, SystemParam},
    math::vec3,
    prelude::*,
};
use bevy_mod_picking::PickableBundle;

use crate::{
    entities::StoreIntoPile,
    systems::{Destructable, FunnyAnimation, Store, Thing},
};

use super::{Boulder, PileSpawn};

#[derive(Clone)]
pub struct Imp {
    pub behavior: ImpBehavior,
    pub idle_time: f32,
    pub idle_new_direction_time: f32,
    pub work_time: f32,
    pub load: Option<Thing>,
    pub load_amount: f32,
    pub walk_destination: WalkDestination,
    pub target_boulder: Target,
    pub target_store: Target,
    pub want_to_follow: Option<Entity>,
    pub idle_complete: bool,
}

#[derive(Clone, Copy, Default)]
pub struct Target {
    entity: Option<Entity>,
    distance_squared: f32,
}

impl Target {
    fn is_near(&self, threshold: f32) -> bool {
        self.entity
            .map_or(false, |_| self.distance_squared <= threshold)
    }
}

impl std::ops::Deref for Target {
    type Target = Option<Entity>;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl Into<WalkDestination> for Target {
    fn into(self) -> WalkDestination {
        self.entity
            .map_or(WalkDestination::None, |e| WalkDestination::Entity(e))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum WalkDestination {
    None,
    Vec3(Vec3),
    Entity(Entity),
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ImpBehavior {
    Idle,
    Dig,
    Store,
    Follow(Entity),
}

impl Imp {
    pub fn new() -> Self {
        Self {
            behavior: ImpBehavior::Idle,
            idle_time: 1.0,
            idle_new_direction_time: 1.0,
            work_time: 0.0,
            load_amount: 0.0,
            load: None,
            walk_destination: WalkDestination::None,
            target_boulder: Target::default(),
            target_store: Target::default(),
            want_to_follow: None,
            idle_complete: false,
        }
    }

    pub fn maybe_follow(&mut self, entity: Entity) {
        self.want_to_follow = Some(entity);
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
            //.add_system(update_imp.label("imp"))
            .add_system(update_imp_commands)
            .add_system_to_stage(CoreStage::PostUpdate, update_walk)
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
                ImpCommands::default(),
                Destructable,
                brain(),
            ))
            .push_children(&[model]);
    }
}

#[derive(SystemParam)]
pub struct QueryBoulders<'w, 's> {
    query: Query<'w, 's, (Entity, &'static Transform, &'static Boulder)>,
}

impl<'w, 's> QueryBoulders<'w, 's> {
    fn get_target_boulder(&self, target: Target, pos: Vec3) -> Target {
        if let Some(entity) = target.entity {
            self.update_target_boulder(entity, pos)
        } else {
            self.find_target_boulder(pos)
        }
    }

    fn find_target_boulder(&self, pos: Vec3) -> Target {
        let mut boulders = Vec::new();

        for (entity, transform, boulder) in self.query.iter() {
            if boulder.marked_for_digging {
                boulders.push(Target {
                    entity: Some(entity),
                    distance_squared: pos.distance_squared(transform.translation),
                });
            }
        }

        if boulders.is_empty() {
            Target::default()
        } else {
            let index = fastrand::usize(0..boulders.len());
            boulders[index]
        }
    }

    fn update_target_boulder(&self, entity: Entity, pos: Vec3) -> Target {
        self.query
            .get(entity)
            .ok()
            .map_or(Target::default(), |(_, transform, _boulder)| {
                // if boulder.marked_for_digging {
                Target {
                    entity: Some(entity),
                    distance_squared: pos.distance_squared(transform.translation),
                }
                // } else {
                // Target::default()
                // }
            })
    }

    fn thing(&self, entity: Entity) -> Option<Thing> {
        self.query
            .get(entity)
            .map(|(_, _, b)| b.material.into())
            .ok()
    }
}

#[derive(SystemParam)]
pub struct QueryStore<'w, 's> {
    query: QuerySet<
        'w,
        's,
        (
            QueryState<(Entity, &'static Transform, &'static Store)>,
            QueryState<&'static mut Store>,
        ),
    >,
}

impl<'w, 's> QueryStore<'w, 's> {
    fn get_target_store(&mut self, target: Target, pos: Vec3, imp: &Imp) -> Target {
        if let Some(load) = imp.load {
            if let Some(entity) = target.entity {
                if let Some((_, transform, store)) = self.query.q0().get(entity).ok() {
                    if store.space_for_thing(load) > 0.0 {
                        return Target {
                            entity: Some(entity),
                            distance_squared: pos.distance_squared(transform.translation),
                        };
                    }
                }
            } else {
                let mut stores: Vec<_> = self
                    .query
                    .q0()
                    .iter()
                    .filter(|(_, _, store)| store.space_for_thing(load) >= 1.0)
                    .collect();
                stores.sort_unstable_by_key(|(_, _, s)| -s.priority_of_thing(load));

                if !stores.is_empty() {
                    let (entity, transform, _) = stores[0];
                    return Target {
                        entity: Some(entity),
                        distance_squared: pos.distance_squared(transform.translation),
                    };
                }
            }
        }

        Target::default()
    }

    fn store_thing(&mut self, entity: Entity, amount: f32, thing: Thing) -> f32 {
        if let Ok(mut store) = self.query.q1().get_mut(entity) {
            store.store_thing(amount, thing)
        } else {
            0.0
        }
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

fn update_imp(
    time: Res<Time>,
    mut cmds: Commands,
    mut imps: Query<(Entity, &mut Imp, &Transform, Option<&FunnyAnimation>)>,
    boulders: QueryBoulders,
    mut stores: QueryStore,
) {
    use ImpBehavior::*;
    let dt = time.delta_seconds();

    for (imp_entity, mut imp, transform, animation) in imps.iter_mut() {
        let pos = transform.translation;

        imp.target_boulder = boulders.get_target_boulder(imp.target_boulder, pos);
        imp.target_store = stores.get_target_store(imp.target_store, pos, &imp);

        let old_behavior = imp.behavior;
        let new_behavior = choose_new_behavior(&imp, old_behavior);

        if old_behavior != new_behavior {
            imp.behavior = new_behavior;

            // leaving old behavior
            match old_behavior {
                Store => {
                    imp.target_store = Target::default();
                    imp.idle_time = 0.0;
                }
                Dig => {
                    imp.target_boulder = Target::default();

                    if animation.is_some() {
                        cmds.entity(imp_entity).remove::<FunnyAnimation>();
                    }
                }
                _ => {}
            }

            // starting new behavior
            match new_behavior {
                Idle => {
                    imp.idle_time = 0.0;
                    imp.idle_new_direction_time = 0.0;
                    imp.walk_destination = WalkDestination::Vec3(pos + random_vec());
                }
                _ => {}
            }
        }

        // handling behavior
        match imp.behavior {
            Idle => {
                if imp.idle_new_direction_time >= 1.0 {
                    imp.idle_new_direction_time = imp.idle_new_direction_time.fract();
                    imp.walk_destination = WalkDestination::Vec3(pos + random_vec());

                    if imp.load_amount > 0.0 {
                        cmds.spawn_bundle((
                            StoreIntoPile {
                                load: imp.load.unwrap(),
                                amount: imp.load_amount,
                                pile: None,
                            },
                            transform.clone(),
                        ));

                        imp.load_amount = 0.0;
                        imp.load = None;
                    }
                }

                imp.idle_time += dt;
                imp.idle_new_direction_time += dt;
                imp.target_boulder = Target::default();
                imp.target_store = Target::default();
            }
            Dig => {
                if imp.target_boulder.is_near(1.0) {
                    imp.work_time += dt;
                    imp.load_amount += dt;

                    // TODO oh, things could change
                    let thing = boulders.thing(imp.target_boulder.unwrap()).unwrap();
                    imp.load = Some(thing);

                    imp.walk_destination = WalkDestination::None;
                    if animation.is_none() {
                        cmds.entity(imp_entity)
                            .insert(FunnyAnimation { offset: 0.0 });
                    }
                } else {
                    if animation.is_some() {
                        cmds.entity(imp_entity).remove::<FunnyAnimation>();
                    }
                    imp.walk_destination = imp.target_boulder.into();
                }
            }
            Store => {
                if imp.target_store.is_near(0.1) {
                    imp.walk_destination = WalkDestination::None;
                    imp.work_time += dt;

                    let store_entity = imp.target_store.entity.unwrap();
                    let thing = imp.load.unwrap();
                    let stored = stores.store_thing(store_entity, imp.load_amount, thing);
                    imp.load_amount -= stored;
                } else {
                    imp.walk_destination = imp.target_store.into();
                }
            }
            Follow(entity) => {
                imp.walk_destination = WalkDestination::Entity(entity);
            }
        }
    }

    fn choose_new_behavior(imp: &Imp, old_behavior: ImpBehavior) -> ImpBehavior {
        if let Some(entity) = imp.want_to_follow {
            Follow(entity)
        } else if imp.idle_time < 1.0 {
            Idle
        } else if imp.target_store.is_some()
            && (imp.load_amount >= 1.0 || imp.load_amount > 0.0 && old_behavior == Store)
        {
            Store
        } else if imp.target_boulder.is_some() && imp.load_amount < 1.0 {
            Dig
        } else {
            Idle
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

#[derive(Default)]
pub struct ImpCommands {
    pub commands: Vec<ImpCommand>,
}

pub enum ImpCommand {
    Dig(Entity),
}

fn update_imp_commands(mut imps: Query<(&mut Imp, &mut ImpCommands)>) {
    for (mut imp, mut imp_cmds) in imps.iter_mut() {
        for cmd in imp_cmds.commands.drain(0..) {
            match cmd {
                ImpCommand::Dig(entity) => {
                    imp.want_to_follow = None;
                    imp.idle_time = 1.0;
                    imp.target_store = Target::default();
                    imp.target_boulder = Target {
                        entity: Some(entity),
                        distance_squared: 1000.0,
                    };
                }
            }
        }
    }
}

use big_brain::prelude::*;

// imp can be Thirsty
// he is as Thirsty as the Thirst tells

struct ImpBrainPlugin;
impl Plugin for ImpBrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(thirsty_scorer_system)
            .add_system(drink_action_system)
            .add_system(thirst_system)
            .add_system(want_to_dig)
            .add_system(want_to_drop)
            .add_system(do_dig)
            .add_system(do_drop)
            .add_system(do_meander)
            .add_plugin(BigBrainPlugin);
    }
}

fn brain() -> ThinkerBuilder {
    // when loaded and storable, store
    // when unloaded and diggable, dig
    // when loaded and not storable, unload
    // default meander

    Thinker::build()
        .picker(FirstToScore { threshold: 0.8 })
        .when(Thirsty::build(), Drink::build())
        .when(WantToStoreBuilder, DoStoreBuilder)
        .when(WantToDigBuilder, DoDigBuilder)
        .when(WantToDropBuilder, DoDropBuilder)
        .otherwise(DoMeanderBuilder)
}

struct WantToStore;
struct WantToDig;
struct WantToDrop;
struct DoStore;
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
        cmd.entity(action).insert(DoStore);
    }
}

#[derive(Debug, Clone)]
struct DoDigBuilder;
impl ActionBuilder for DoDigBuilder {
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action).insert(DoDig);
    }
}

#[derive(Debug, Clone)]
struct DoDropBuilder;
impl ActionBuilder for DoDropBuilder {
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action).insert(DoDrop);
    }
}

#[derive(Debug, Clone)]
struct DoMeanderBuilder;
impl ActionBuilder for DoMeanderBuilder {
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action).insert(DoMeander);
    }
}

////////

fn want_to_drop(imps: Query<&Imp>, mut query: Query<(&Actor, &mut Score), With<WantToDrop>>) {
    for (Actor(actor), mut score) in query.iter_mut() {
        if let Ok(imp) = imps.get(*actor) {
            if imp.load_amount >= 1.0 {
                score.set(1.0);
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
                    println!("DoDrop");
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
) {
    let dt = time.delta_seconds();

    for (Actor(actor), mut state) in query.iter_mut() {
        if let Ok((mut imp, transform)) = imps.get_mut(*actor) {
            let pos = transform.translation;

            match *state {
                ActionState::Requested => {
                    println!("DoDig");
                    imp.walk_destination = boulders
                        .iter()
                        .filter(|(_, boulder, _)| boulder.marked_for_digging)
                        .filter(|(_, boulder, _)| {
                            imp.load.is_none() || imp.load == Some(boulder.material.into())
                        })
                        .map(|(entity, _, transform)| {
                            (entity, pos.distance_squared(transform.translation))
                        })
                        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Less))
                        .map(|(e, _)| WalkDestination::Entity(e))
                        .unwrap_or(WalkDestination::None);
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    if let WalkDestination::Entity(entity) = imp.walk_destination {
                        if let Ok((_, boulder, transform)) = boulders.get(entity) {
                            let material = Some(boulder.material.into());

                            if imp.load.is_some() && imp.load != material {
                                *state = ActionState::Failure;
                            } else if imp.load_amount < 1.0 {
                                if imp.load.is_none() {
                                    imp.load = material;
                                }

                                if pos.distance_squared(transform.translation) < 1.0 {
                                    imp.load_amount = (imp.load_amount + dt).min(1.0);
                                }
                            } else {
                                *state = ActionState::Success;
                            }
                        }
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
                    imp.walk_destination = WalkDestination::None;
                }
                _ => {}
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
                ActionState::Requested => {
                    println!("DoMeander");
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
                _ => {}
            }
        }
    }
}

////////

struct Thirst {
    thirst: f32,
    per_second: f32,
}

fn thirst_system(time: Res<Time>, mut thirsts: Query<&mut Thirst>) {
    for mut thirst in thirsts.iter_mut() {
        thirst.thirst += thirst.per_second * (time.delta().as_micros() as f32 / 1_000_000.0);
        if thirst.thirst >= 100.0 {
            thirst.thirst = 100.0;
        }
        println!("Thirst: {}", thirst.thirst);
    }
}

impl Thirst {
    fn new(thirst: f32, per_second: f32) -> Self {
        Self { thirst, per_second }
    }
}

#[derive(Debug, Clone)]
struct Thirsty;

impl Thirsty {
    fn build() -> ThirstyBuilder {
        ThirstyBuilder
    }
}

#[derive(Debug, Clone)]
struct ThirstyBuilder;

impl ScorerBuilder for ThirstyBuilder {
    fn build(&self, cmd: &mut Commands, scorer: Entity, _actor: Entity) {
        cmd.entity(scorer).insert(Thirsty);
    }
}

fn thirsty_scorer_system(
    thirsts: Query<&Thirst>,
    mut query: Query<(&Actor, &mut Score), With<Thirsty>>,
) {
    for (Actor(actor), mut score) in query.iter_mut() {
        if let Ok(thirst) = thirsts.get(*actor) {
            score.set(thirst.thirst / 100.0);
        }
    }
}

#[derive(Debug, Clone)]
struct Drink;

impl Drink {
    fn build() -> DrinkBuilder {
        DrinkBuilder
    }
}

#[derive(Debug, Clone)]
struct DrinkBuilder;

impl ActionBuilder for DrinkBuilder {
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action).insert(Drink);
    }
}

fn drink_action_system(
    mut thirsts: Query<&mut Thirst>,
    mut query: Query<(&Actor, &mut ActionState), With<Drink>>,
) {
    for (Actor(actor), mut state) in query.iter_mut() {
        if let Ok(mut thirst) = thirsts.get_mut(*actor) {
            match *state {
                ActionState::Requested => {
                    println!("Drink");
                    thirst.thirst = 10.0;
                    *state = ActionState::Success;
                }
                ActionState::Cancelled => {
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        }
    }
}

///////

fn random_vec() -> Vec3 {
    let a = TAU * fastrand::f32();
    vec3(a.cos(), 0.0, a.sin())
}
