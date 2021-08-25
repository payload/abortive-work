use crate::systems::*;
use bevy::{ecs::system::SystemParam, prelude::*};

use super::{Blocking, NotGround};

pub struct Smithery {
    work_time: f32,
    working: bool,
}

impl Smithery {
    pub fn new() -> Self {
        Self {
            work_time: 0.0,
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
    mut smitheries: Query<(Entity, &mut Smithery, &mut Store, Option<&FunnyAnimation>)>,
    time: Res<Time>,
    mut cmds: Commands,
) {
    let dt = time.delta_seconds();

    for (entity, mut smithery, mut store, animation) in smitheries.iter_mut() {
        let coal = store.amount(0);
        let iron = store.amount(1);

        if !smithery.working && coal >= 1.0 && iron >= 1.0 {
            smithery.working = true;
            smithery.work_time = 0.0;
        } else if smithery.working && (coal == 0.0 || iron == 0.0) {
            smithery.working = false;
        }

        if smithery.working {
            smithery.work_time += dt;

            if animation.is_none() {
                cmds.entity(entity).insert(FunnyAnimation { offset: 0.0 });
            }

            if smithery.work_time >= 1.0 {
                smithery.working = false;

                if store.space_for_thing(Thing::Tool) >= 1.0 {
                    smithery.work_time -= 1.0;
                    store.decrease(0, 1.0);
                    store.decrease(1, 1.0);
                    store.store(2, 1.0, Thing::Tool);
                }
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
                Store::new(&[
                    StoreSlot::input(5.0, Thing::Coal.into()),
                    StoreSlot::input(5.0, Thing::Iron.into()),
                    StoreSlot::output(1.0, Thing::Tool.into()),
                ]),
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
            base_color: Color::DARK_GRAY,
            ..Default::default()
        }),
        mesh: meshes.add(shape::Box::new(1.2, 0.8, 1.2).into()),
    });
}
