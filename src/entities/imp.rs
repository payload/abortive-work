use std::{cmp::Ordering, f32::consts::TAU};

use bevy::{ecs::system::SystemParam, math::vec3, prelude::*};
use bevy_mod_picking::PickableBundle;
use bevy_prototype_debug_lines::DebugLines;

use crate::{
    entities::StoreIntoPile,
    systems::{DebugConfig, Destructable, FunnyAnimation, Thing},
};

use super::Boulder;

#[derive(Clone)]
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
}

#[derive(Clone, Copy, Debug)]
pub enum WalkDestination {
    None,
    Vec3(Vec3),
    Entity(Entity),
}

impl Imp {
    pub fn new() -> Self {
        Self {
            idle_time: 1.0,
            idle_new_direction_time: 1.0,
            work_time: 0.0,
            load_amount: 0.0,
            load: None,
            walk_destination: WalkDestination::None,
            want_to_follow: None,
            idle_complete: false,
            boulder: None,
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
            .add_system(update_imp_commands)
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
                ImpCommands::default(),
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
                ImpCommand::Dig(_entity) => {
                    imp.want_to_follow = None;
                    imp.idle_time = 1.0;
                }
            }
        }
    }
}

use big_brain::prelude::*;

struct ImpBrainPlugin;
impl Plugin for ImpBrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(want_to_dig)
            .add_system(want_to_drop)
            .add_system(do_dig)
            .add_system(do_drop)
            .add_system(do_meander)
            .add_plugin(BigBrainPlugin);
    }
}

fn brain() -> ThinkerBuilder {
    Thinker::build()
        .picker(FirstToScore { threshold: 0.8 })
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

                    imp.walk_destination = imp
                        .boulder
                        .map(|e| WalkDestination::Entity(e))
                        .unwrap_or(WalkDestination::None);

                    cmds.entity(*actor).insert(FunnyAnimation { offset: 0.0 });
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
