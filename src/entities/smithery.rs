use crate::systems::*;
use bevy::{ecs::system::SystemParam, prelude::*};

use super::{Blocking, NotGround, Rock, Storage};

pub struct Smithery {
    coal: f32,
    work_time: f32,
    tools: u32,
    working: bool,
}

impl Smithery {
    pub fn new() -> Self {
        Self {
            coal: 1.0,
            work_time: 0.0,
            tools: 0,
            working: false,
        }
    }
}

pub struct SmitheryPlugin;

impl Plugin for SmitheryPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets)
            .add_system(update);
    }
}

fn update(
    mut smitheries: Query<(Entity, &mut Smithery, &mut Storage, Option<&FunnyAnimation>)>,
    time: Res<Time>,
    mut cmds: Commands,
) {
    let dt = time.delta_seconds();

    for (entity, mut smithery, mut storage, animation) in smitheries.iter_mut() {
        let iron = storage.rock.amount;

        if !smithery.working && smithery.coal >= 1.0 && iron >= 1.0 {
            smithery.working = true;
        } else if smithery.working && (smithery.coal == 0.0 || iron == 0.0) {
            smithery.working = false;
        }

        // if !smithery.working {
        //     smithery.coal += dt * 0.5;
        //     smithery.iron += dt * 0.5;
        // }

        if smithery.working {
            // smithery.coal -= dt;
            storage.rock.amount -= dt;
            smithery.work_time += dt;

            if animation.is_none() {
                cmds.entity(entity).insert(FunnyAnimation { offset: 0.0 });
            }

            if smithery.work_time >= 1.0 {
                smithery.work_time -= 1.0;
                smithery.tools += 1;
                println!("Tools {}", smithery.tools);
                smithery.working = false;
            }
        } else {
            if animation.is_some() {
                cmds.entity(entity).remove::<FunnyAnimation>();
            }
        }
    }
}

#[derive(SystemParam)]
pub struct SmitherySpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, SmitheryAssets>,
}

impl<'w, 's> SmitherySpawn<'w, 's> {
    pub fn spawn(&mut self, smithery: Smithery, transform: Transform) {
        let model = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: self.assets.transform.clone(),
                material: self.assets.material.clone(),
                mesh: self.assets.mesh.clone(),
                ..Default::default()
            })
            .insert(NotGround)
            .insert(Blocking)
            .id();

        self.cmds
            .spawn_bundle((
                smithery,
                transform,
                GlobalTransform::identity(),
                Storage {
                    prio: 2,
                    rock: Rock {
                        amount: 0.0,
                        material: super::BoulderMaterial::Iron,
                    },
                },
            ))
            .push_children(&[model]);
    }
}

#[derive(Clone)]
pub struct SmitheryAssets {
    pub transform: Transform,
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
}

fn load_assets(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.insert_resource(SmitheryAssets {
        transform: Transform::from_xyz(0.0, 0.4, 0.0),
        material: materials.add(StandardMaterial {
            unlit: true,
            base_color: Color::DARK_GRAY,
            ..Default::default()
        }),
        mesh: meshes.add(shape::Box::new(1.2, 0.8, 1.2).into()),
    });
}
