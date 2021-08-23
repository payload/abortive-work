use std::f32::consts::TAU;

use bevy::{ecs::system::SystemParam, math::vec3, prelude::*};

use crate::systems::FunnyAnimation;

use super::{Boulder, Storage};

pub struct Imp {
    behavior: ImpBehavior,
    idle_time: f32,
    idle_new_direction_time: f32,
    work_time: f32,
    loaded_rock: f32,
    walk_destination: WalkDestination,
    target_boulder: Target,
    target_storage: Target,
}

#[derive(Clone, Copy, Default)]
struct Target {
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

enum WalkDestination {
    None,
    Vec3(Vec3),
    Entity(Entity),
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum ImpBehavior {
    Idle,
    Dig,
    Store,
}

impl Imp {
    pub fn new() -> Self {
        Self {
            behavior: ImpBehavior::Idle,
            idle_time: 1.0,
            idle_new_direction_time: 1.0,
            work_time: 0.0,
            loaded_rock: 0.0,
            walk_destination: WalkDestination::None,
            target_boulder: Target::default(),
            target_storage: Target::default(),
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
            .add_system(update_imp.label("imp"))
            .add_system(update_walk.after("imp"));
    }
}

#[derive(SystemParam)]
pub struct ImpSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, ImpAssets>,
}

impl<'w, 's> ImpSpawn<'w, 's> {
    pub fn spawn(&mut self, imp: Imp, transform: Transform) {
        let pbr_bundle = PbrBundle {
            transform: self.assets.transform.clone(),
            material: self.assets.material.clone(),
            mesh: self.assets.mesh.clone(),
            ..Default::default()
        };

        self.cmds
            .spawn_bundle((imp, transform, GlobalTransform::identity()))
            .with_children(|p| {
                p.spawn_bundle(pbr_bundle);
            });
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
            .map_or(Target::default(), |(_, transform, boulder)| {
                if boulder.marked_for_digging {
                    Target {
                        entity: Some(entity),
                        distance_squared: pos.distance_squared(transform.translation),
                    }
                } else {
                    Target::default()
                }
            })
    }
}

#[derive(SystemParam)]
pub struct QueryStorages<'w, 's> {
    query: Query<'w, 's, (Entity, &'static Transform, &'static Storage)>,
}

impl<'w, 's> QueryStorages<'w, 's> {
    fn get_target_storage(&self, target: Target, pos: Vec3) -> Target {
        if let Some(entity) = target.entity {
            self.query
                .get(entity)
                .ok()
                .map_or(Target::default(), |(_, transform, _storage)| Target {
                    entity: Some(entity),
                    distance_squared: pos.distance_squared(transform.translation),
                })
        } else {
            let vec: Vec<_> = self.query.iter().collect();

            if !vec.is_empty() {
                let index = fastrand::usize(0..vec.len());
                let (entity, transform, _storage) = vec[index];
                Target {
                    entity: Some(entity),
                    distance_squared: pos.distance_squared(transform.translation),
                }
            } else {
                Target::default()
            }
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
            unlit: true,
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
    storages: QueryStorages,
) {
    use ImpBehavior::*;
    let dt = time.delta_seconds();

    for (imp_entity, mut imp, transform, animation) in imps.iter_mut() {
        let pos = transform.translation;

        imp.target_boulder = boulders.get_target_boulder(imp.target_boulder, pos);
        imp.target_storage = storages.get_target_storage(imp.target_storage, pos);

        let old_behavior = imp.behavior;
        let new_behavior = if imp.idle_time < 1.0 {
            Idle
        } else if imp.target_storage.is_some()
            && (imp.loaded_rock >= 1.0 || imp.loaded_rock > 0.0 && old_behavior == Store)
        {
            Store
        } else if imp.target_boulder.is_some() && imp.loaded_rock < 1.0 {
            Dig
        } else {
            imp.idle_time = 0.0;
            Idle
        };

        if old_behavior != new_behavior {
            imp.behavior = new_behavior;

            match old_behavior {
                Store => {
                    imp.idle_time = 0.0;
                }
                Dig => {
                    if animation.is_some() {
                        cmds.entity(imp_entity).remove::<FunnyAnimation>();
                    }
                }
                _ => {}
            }

            match new_behavior {
                Idle => {
                    imp.idle_new_direction_time = 0.0;
                    imp.walk_destination = WalkDestination::Vec3(pos + random_vec());
                }
                _ => {}
            }
        }

        match imp.behavior {
            Idle => {
                if imp.idle_new_direction_time >= 1.0 {
                    imp.idle_new_direction_time = imp.idle_new_direction_time.fract();
                    imp.walk_destination = WalkDestination::Vec3(pos + random_vec());
                }

                imp.idle_time += dt;
                imp.idle_new_direction_time += dt;
                imp.target_boulder = Target::default();
                imp.target_storage = Target::default();
            }
            Dig => {
                if imp.target_boulder.is_near(1.0) {
                    imp.work_time += dt;
                    imp.loaded_rock += dt;
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
                if imp.target_storage.is_near(0.1) {
                    imp.work_time += dt;
                    imp.loaded_rock = (imp.loaded_rock - dt).max(0.0);
                    imp.walk_destination = WalkDestination::None;
                } else {
                    imp.walk_destination = imp.target_storage.into();
                }
            }
        }
    }

    fn random_vec() -> Vec3 {
        let a = TAU * fastrand::f32();
        vec3(a.cos(), 0.0, a.sin())
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
