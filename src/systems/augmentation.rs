use bevy::{
    ecs::system::{EntityCommands, SystemParam},
    prelude::*,
};

use super::{disk, ring};

pub struct AugmentationPlugin;

impl Plugin for AugmentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets)
            .add_system_to_stage(CoreStage::PostUpdate, update_pedestals);
    }
}

#[derive(SystemParam)]
pub struct AugmentSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, AugmentAssets>,
    with_pedestal: Query<'w, 's, &'static WithPedestal>,
    transform: Query<'w, 's, &'static Transform>,
    meshes: ResMut<'w, Assets<Mesh>>,
}

impl<'w, 's> AugmentSpawn<'w, 's> {
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
            let pedestal = self.spawn_pedestal(Pedestal(entity), pos).id();
            self.cmds.entity(entity).insert(WithPedestal(pedestal));
        }
    }

    pub fn remove_pedestal(&mut self, entity: Entity) {
        if let Ok(WithPedestal(pedestal)) = self.with_pedestal.get(entity) {
            self.cmds.entity(entity).remove::<WithPedestal>();
            self.cmds.entity(*pedestal).despawn_recursive();
        }
    }

    pub fn spawn_pedestal<'a>(
        &'a mut self,
        pedestal: Pedestal,
        pos: Vec3,
    ) -> EntityCommands<'w, 's, 'a> {
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
            pedestal,
        ));
        entity_cmds.push_children(&[model]);
        entity_cmds
    }

    #[allow(unused)]
    pub fn spawn_ring<'a>(&'a mut self, pos: Vec3, radius: f32) -> EntityCommands<'w, 's, 'a> {
        let e_cmds = self.cmds.spawn_bundle(PbrBundle {
            transform: Transform::from_translation(pos),
            material: self.assets.pedestal_material.clone(),
            mesh: self.meshes.add(ring(radius + 0.1, radius, 24)),
            ..Default::default()
        });
        e_cmds
    }

    pub fn spawn_disk<'a>(&'a mut self, pos: Vec3, radius: f32) -> EntityCommands<'w, 's, 'a> {
        let e_cmds = self.cmds.spawn_bundle(PbrBundle {
            transform: Transform::from_translation(pos),
            material: self.assets.pedestal_material.clone(),
            mesh: self.meshes.add(disk(radius + 0.1, 24)),
            ..Default::default()
        });
        e_cmds
    }
}

#[derive(Clone)]
pub struct AugmentAssets {
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

pub struct Pedestal(Entity);
pub struct WithPedestal(Entity);

fn update_pedestals(
    augmented: Query<&Transform, With<WithPedestal>>,
    mut pedestals: Query<(Entity, &mut Transform, &Pedestal), Without<WithPedestal>>,
    mut cmds: Commands,
) {
    for (entity, mut transform, pedestal) in pedestals.iter_mut() {
        if let Ok(augmented_transform) = augmented.get(pedestal.0) {
            let a = augmented_transform.translation;
            let b = transform.translation;
            if a.x != b.x || a.z != b.z {
                transform.translation.x = a.x;
                transform.translation.z = a.z;
            }
        } else {
            cmds.entity(entity).despawn_recursive();
        }
    }
}
