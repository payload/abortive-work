use std::f32::consts::TAU;

use bevy::{ecs::system::SystemParam, math::vec3, prelude::*};

use crate::systems::FunnyAnimation;

use super::{Boulder, Storage};

pub struct Imp {
    behavior: ImpBehavior,
    walk_to: Vec3,
}

enum ImpBehavior {
    Idle { time: f32 },
    WalkToDig { boulder: Entity },
    Dig { boulder: Entity, time: f32 },
    WalkToStore { storage: Entity },
    Store { storage: Entity, time: f32 },
}

impl Imp {
    pub fn new() -> Self {
        Self {
            behavior: ImpBehavior::Idle { time: 0.0 },
            walk_to: Vec3::ZERO,
        }
    }
}

#[derive(Clone)]
pub struct ImpAssets {
    transform: Transform,
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
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
    mut imps: Query<(Entity, &mut Imp, &Transform)>,
    boulders: Query<(Entity, &Transform, &Boulder)>,
    storages: Query<(Entity, &Transform, &Storage)>,
) {
    use ImpBehavior::*;
    let now = time.time_since_startup().as_secs_f32();
    let dt = time.delta_seconds();

    for (entity, mut imp, transform) in imps.iter_mut() {
        match imp.behavior {
            Idle { time } => {
                if time <= now {
                    let _ =
                        to_walk_to_dig(&mut imp, &boulders) || to_idle(now, transform, &mut imp);
                }
            }
            WalkToDig { boulder } => {
                if let Ok((_, boulder_transform, boulder_component)) = boulders.get(boulder) {
                    let digging = boulder_component.marked_for_digging;
                    let is_far = || {
                        boulder_transform
                            .translation
                            .distance_squared(transform.translation)
                            > 1.0
                    };

                    if digging && is_far() {
                        imp.walk_to = boulder_transform.translation;
                    } else if digging {
                        imp.walk_to = transform.translation;
                        imp.behavior = Dig { boulder, time: 0.0 };
                        cmds.entity(entity).insert(FunnyAnimation { offset: 0.0 });
                    } else {
                        to_idle(now, transform, &mut imp);
                    }
                } else {
                    to_idle(now, transform, &mut imp);
                }
            }
            Dig { boulder, time } => {
                let mut leave_state = || {
                    cmds.entity(entity).remove::<FunnyAnimation>();
                };

                if let Ok((_, boulder_transform, boulder_component)) = boulders.get(boulder) {
                    let digging = boulder_component.marked_for_digging;
                    let is_far = || {
                        boulder_transform
                            .translation
                            .distance_squared(transform.translation)
                            > 1.0
                    };

                    if digging && !is_far() {
                        let time = time + dt;

                        if time >= 1.5 {
                            leave_state();
                            let _ = to_walk_to_store(&mut imp, &storages)
                                || to_idle(now, transform, &mut imp);
                        } else {
                            imp.behavior = Dig { boulder, time };
                        }
                    } else if !digging {
                        leave_state();
                        to_idle(now, transform, &mut imp);
                    }
                } else {
                    leave_state();
                    to_idle(now, transform, &mut imp);
                }
            }
            WalkToStore { storage } => {
                if let Ok((_, t, _)) = storages.get(storage) {
                    if t.translation.distance_squared(transform.translation) < 0.3 {
                        imp.behavior = Store {
                            storage,
                            time: now + 0.8,
                        };
                    }
                } else {
                    let _ =
                        to_walk_to_store(&mut imp, &storages) || to_idle(now, transform, &mut imp);
                }
            }
            Store { storage, time } => {
                if let Ok((_, t, _)) = storages.get(storage) {
                    if t.translation.distance_squared(transform.translation) < 0.3 {
                        if time <= now {
                            to_idle(now, transform, &mut imp);
                        }
                    } else {
                        imp.behavior = WalkToStore { storage };
                    }
                } else {
                    let _ =
                        to_walk_to_store(&mut imp, &storages) || to_idle(now, transform, &mut imp);
                }
            }
        }
    }

    fn to_idle(now: f32, transform: &Transform, imp: &mut Imp) -> bool {
        let a = TAU * fastrand::f32();
        let random_offset = vec3(a.cos(), 0.0, a.sin());
        imp.walk_to = transform.translation + random_offset;
        imp.behavior = Idle { time: now + 1.0 };
        true
    }

    fn to_walk_to_dig(imp: &mut Imp, boulders: &Query<(Entity, &Transform, &Boulder)>) -> bool {
        if let Some((boulder, _walk_to)) = diggable_boulder(boulders) {
            imp.behavior = WalkToDig { boulder };
            true
        } else {
            false
        }
    }

    fn to_walk_to_store(imp: &mut Imp, storages: &Query<(Entity, &Transform, &Storage)>) -> bool {
        let vec: Vec<_> = storages.iter().collect();

        if !vec.is_empty() {
            let index = fastrand::usize(0..vec.len());
            let (entity, t, _) = vec[index];
            imp.behavior = WalkToStore { storage: entity };
            imp.walk_to = t.translation;
            true
        } else {
            false
        }
    }

    fn diggable_boulder(query: &Query<(Entity, &Transform, &Boulder)>) -> Option<(Entity, Vec3)> {
        let mut boulders: Vec<(Entity, Vec3)> = Vec::new();

        for (entity, transform, boulder) in query.iter() {
            if boulder.marked_for_digging {
                boulders.push((entity, transform.translation));
            }
        }

        if boulders.is_empty() {
            None
        } else {
            let index = fastrand::usize(0..boulders.len());
            Some(boulders[index])
        }
    }
}

fn update_walk(time: Res<Time>, mut imps: Query<(&Imp, &mut Transform)>) {
    let dt = time.delta_seconds();

    for (imp, mut transform) in imps.iter_mut() {
        let diff = imp.walk_to - transform.translation;
        let len2 = diff.length_squared();
        let vec = if len2 < 1.0 { diff } else { diff / len2.sqrt() };
        let speed = 3.0;
        let step = vec * speed * dt;
        transform.translation += step;
    }
}
