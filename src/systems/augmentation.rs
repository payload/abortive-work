use std::{f32::consts::PI, f32::consts::TAU};

use bevy::{
    ecs::{
        entity::EntityMap,
        system::{EntityCommands, SystemParam},
    },
    prelude::*,
};

use super::disk;

pub struct AugmentationPlugin;

impl Plugin for AugmentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets)
            .add_system(coin_animation)
            .add_system_to_stage(CoreStage::PostUpdate, update_transform)
            .insert_resource(AugmentState::default());
    }
}

#[derive(Default)]
pub struct AugmentState {
    coins: EntityMap,
}

#[derive(SystemParam)]
pub struct AugmentSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, AugmentAssets>,
    state: ResMut<'w, AugmentState>,
    with_pedestal: Query<'w, 's, &'static WithPedestal>,
    transform: Query<'w, 's, &'static Transform>,
}

impl<'w, 's> AugmentSpawn<'w, 's> {
    pub fn add_coin(&mut self, entity: Entity) {
        if self.state.coins.get(entity).is_err() {
            let coin = self.spawn_coin().id();
            self.cmds.entity(entity).push_children(&[coin]);
            self.state.coins.insert(entity, coin);
        }
    }

    pub fn remove_coin(&mut self, entity: Entity) {
        for coin in self.state.coins.get(entity) {
            self.cmds.entity(coin).despawn_recursive();
            self.state.coins.remove(entity);
        }
    }

    pub fn with_pedestal(&mut self, entity: Entity, enabled: bool) {
        if enabled {
            self.add_pedestal(entity);
        } else {
            self.remove_pedestal(entity);
        }
    }

    pub fn add_pedestal(&mut self, entity: Entity) {
        if self.with_pedestal.get(entity).is_err() {
            let pos = self.transform.get(entity).unwrap().translation;
            let pedestal = self.spawn_pedestal(pos).id();
            self.cmds.entity(entity).insert(WithPedestal(pedestal));
        }
    }

    pub fn remove_pedestal(&mut self, entity: Entity) {
        if let Ok(WithPedestal(pedestal)) = self.with_pedestal.get(entity) {
            self.cmds.entity(entity).remove::<WithPedestal>();
            self.cmds.entity(*pedestal).despawn_recursive();
        }
    }

    pub fn spawn_coin<'a>(&'a mut self) -> EntityCommands<'w, 's, 'a> {
        let model = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: self.assets.coin_transform.clone(),
                material: self.assets.coin_material.clone(),
                mesh: self.assets.coin_mesh.clone(),
                ..Default::default()
            })
            .id();

        let mut entity_cmds = self.cmds.spawn_bundle((
            Transform::from_xyz(0.0, 2.0, 0.0),
            GlobalTransform::identity(),
        ));
        entity_cmds.push_children(&[model]).insert(CoinAnimation);
        entity_cmds
    }

    pub fn spawn_pedestal<'a>(&'a mut self, pos: Vec3) -> EntityCommands<'w, 's, 'a> {
        let model = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: self.assets.pedestal_transform.clone(),
                material: self.assets.pedestal_material.clone(),
                mesh: self.assets.pedestal_mesh.clone(),
                ..Default::default()
            })
            .id();

        let mut entity_cmds = self.cmds.spawn_bundle((
            Transform::from_xyz(pos.x, 0.0, pos.z),
            GlobalTransform::identity(),
            Pedestal,
        ));
        entity_cmds.push_children(&[model]);
        entity_cmds
    }
}

#[derive(Clone)]
pub struct AugmentAssets {
    pub coin_transform: Transform,
    pub coin_material: Handle<StandardMaterial>,
    pub coin_mesh: Handle<Mesh>,

    pub pedestal_transform: Transform,
    pub pedestal_material: Handle<StandardMaterial>,
    pub pedestal_mesh: Handle<Mesh>,
}

fn load_assets(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.insert_resource(AugmentAssets {
        coin_transform: Transform::from_rotation(Quat::from_rotation_z(0.5 * PI)),
        coin_material: materials.add(color_material(Color::ORANGE)),
        coin_mesh: meshes.add(shape::Box::new(0.2, 0.0, 0.2).into()),

        pedestal_transform: Transform::from_xyz(0.0, 0.05, 0.0),
        pedestal_material: materials.add(color_material(Color::ANTIQUE_WHITE)),
        pedestal_mesh: meshes.add(disk(0.9, 24)),
    });

    fn color_material(color: Color) -> StandardMaterial {
        StandardMaterial {
            base_color: color,
            unlit: true,
            ..Default::default()
        }
    }
}

struct CoinAnimation;

fn coin_animation(mut query: Query<&mut Transform, With<CoinAnimation>>, time: Res<Time>) {
    let dt = time.delta_seconds();
    let angle = TAU * dt;

    for mut transform in query.iter_mut() {
        transform.rotate(Quat::from_rotation_y(angle));
    }
}

struct Pedestal;
pub struct WithPedestal(Entity);

fn update_transform(
    augmented: Query<(&Transform, &WithPedestal), Changed<Transform>>,
    mut augments: Query<&mut Transform, (With<Pedestal>, Without<WithPedestal>)>,
    mut cmds: Commands,
) {
    for (changed, WithPedestal(entity)) in augmented.iter() {
        if let Ok(mut transform) = augments.get_mut(*entity) {
            transform.translation.x = changed.translation.x;
            transform.translation.z = changed.translation.z;
        } else {
            cmds.entity(*entity).remove::<WithPedestal>();
        }
    }
}
