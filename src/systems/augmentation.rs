use std::{f32::consts::PI, f32::consts::TAU};

use bevy::{
    ecs::{
        entity::EntityMap,
        system::{EntityCommands, SystemParam},
    },
    prelude::*,
};

use super::disk;

pub enum Augment {
    Coin,
    Pedestal,
}

pub struct AugmentationPlugin;

impl Plugin for AugmentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets)
            .add_system_to_stage(CoreStage::PostUpdate, update_augmentation)
            .add_system(coin_animation);
    }
}

fn update_augmentation(
    mut cmds: Commands,
    mut map: Local<EntityMap>,
    added: Query<(Entity, &Augment), Added<Augment>>,
    removed: RemovedComponents<Augment>,
    mut spawn: AugmentSpawn,
) {
    for (entity, augment) in added.iter() {
        let augment_entity = match augment {
            Augment::Coin => spawn.spawn_coin().id(),
            Augment::Pedestal => spawn.spawn_pedestal().id(),
        };

        cmds.entity(entity).push_children(&[augment_entity]);
        map.insert(entity, augment_entity);
    }

    for entity in removed.iter() {
        if let Ok(augment_entity) = map.get(entity) {
            cmds.entity(augment_entity).despawn_recursive();
        }
    }
}

#[derive(SystemParam)]
pub struct AugmentSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, AugmentAssets>,
}

impl<'w, 's> AugmentSpawn<'w, 's> {
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
            Transform::from_xyz(0.0, 1.0, 0.0),
            GlobalTransform::identity(),
        ));
        entity_cmds.push_children(&[model]).insert(CoinAnimation);
        entity_cmds
    }

    pub fn spawn_pedestal<'a>(&'a mut self) -> EntityCommands<'w, 's, 'a> {
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
            Transform::from_xyz(0.0, 0.0005, 0.0),
            GlobalTransform::identity(),
        ));
        entity_cmds.push_children(&[model]).insert(CoinAnimation);
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
        coin_mesh: meshes.add(disk(0.2, 12)),

        pedestal_transform: Transform::identity(),
        pedestal_material: materials.add(color_material(Color::ANTIQUE_WHITE)),
        pedestal_mesh: meshes.add(disk(0.6, 24)),
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
