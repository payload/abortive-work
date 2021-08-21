use std::f32::consts::TAU;

use bevy::{ecs::system::SystemParam, math::vec3, prelude::*};

use crate::systems::FunnyAnimation;

use super::Boulder;

pub struct Imp {
    behavior: ImpBehavior,
    walk_to: Vec3,
}

enum ImpBehavior {
    Idle { time: f32 },
    Dig { boulder: Entity, time: f32 },
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
) {
    use ImpBehavior::*;
    let now = time.time_since_startup().as_secs_f32();
    let dt = time.delta_seconds();

    for (entity, mut imp, transform) in imps.iter_mut() {
        match imp.behavior {
            Idle { time } => {
                if time <= now {
                    let _ = to_dig(&mut imp, &boulders) || to_idle(now, transform, &mut imp);
                }
            }
            Dig { boulder, time } => {
                let mut imp_cmds = cmds.entity(entity);
                let mut leave_state = || {
                    imp_cmds.remove::<FunnyAnimation>();
                };

                if let Ok((_, boulder_transform, boulder_component)) = boulders.get(boulder) {
                    if !boulder_component.marked_for_digging {
                        leave_state();
                        to_idle(now, transform, &mut imp);
                    } else if boulder_transform
                        .translation
                        .distance_squared(transform.translation)
                        < 1.0
                    {
                        let time = time + dt;

                        if time >= 1.5 {
                            leave_state();
                            to_idle(now, transform, &mut imp);
                        } else {
                            imp.walk_to = transform.translation;
                            imp.behavior = Dig { boulder, time };
                            imp_cmds.insert(FunnyAnimation { offset: 0.0 });
                        }
                    }
                } else {
                    leave_state();
                    to_idle(now, transform, &mut imp);
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

    fn to_dig(imp: &mut Imp, boulders: &Query<(Entity, &Transform, &Boulder)>) -> bool {
        if let Some((boulder, walk_to)) = diggable_boulder(boulders) {
            imp.behavior = Dig { boulder, time: 0.0 };
            imp.walk_to = walk_to;
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
