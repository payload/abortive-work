use bevy::{ecs::system::SystemParam, prelude::*};

use crate::systems::*;

use super::{Blocking, NotGround};

#[derive(Clone)]
pub struct Boulder {
    pub material: BoulderMaterial,
    pub marked_for_digging: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoulderMaterial {
    Stone,
    Coal,
    Iron,
    Gold,
}

pub struct BoulderModel;

impl Boulder {
    pub fn new(material: BoulderMaterial) -> Self {
        Self {
            material,
            marked_for_digging: false,
        }
    }
}

#[derive(Clone)]
pub struct BoulderAssets {
    pub transform: Transform,
    pub stone: Handle<StandardMaterial>,
    pub coal: Handle<StandardMaterial>,
    pub iron: Handle<StandardMaterial>,
    pub gold: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
}

pub struct BoulderPlugin;

impl Plugin for BoulderPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::Startup, load_boulder_assets)
            .add_system_to_stage(CoreStage::PreUpdate, update_boulder_config)
            .init_resource::<BoulderConfig>();
    }
}

pub struct BoulderConfig {
    pub max_angle_deviation: f32,
}

impl Default for BoulderConfig {
    fn default() -> Self {
        Self {
            max_angle_deviation: 8.0_f32.to_radians(),
        }
    }
}

#[derive(SystemParam)]
pub struct BoulderSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, BoulderAssets>,
    config: Res<'w, BoulderConfig>,
}

impl<'w, 's> BoulderSpawn<'w, 's> {
    pub fn spawn(&mut self, boulder: Boulder, mut transform: Transform) {
        transform.translation.y = -0.2 * fastrand::f32();
        transform.rotate(
            Quat::from_rotation_z(self.config.max_angle_deviation * (0.5 - fastrand::f32()))
                .mul_quat(Quat::from_rotation_x(
                    self.config.max_angle_deviation * (0.5 - fastrand::f32()),
                )),
        );

        let model = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: self.assets.transform,
                material: match boulder.material {
                    BoulderMaterial::Stone => self.assets.stone.clone(),
                    BoulderMaterial::Coal => self.assets.coal.clone(),
                    BoulderMaterial::Iron => self.assets.iron.clone(),
                    BoulderMaterial::Gold => self.assets.gold.clone(),
                },
                mesh: self.assets.mesh.clone(),
                ..Default::default()
            })
            .insert(NotGround)
            .insert(Blocking)
            .insert(BoulderModel)
            .id();

        self.cmds
            .spawn_bundle((
                boulder,
                transform,
                GlobalTransform::identity(),
                Destructable,
                FocusObject::new(),
            ))
            .push_children(&[model]);
    }
}

fn load_boulder_assets(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<ThingMaterials>,
) {
    cmds.insert_resource(BoulderAssets {
        transform: Transform::from_xyz(0.0, 0.9, 0.0),
        mesh: meshes.add(shape::Box::new(1.0, 2.0, 1.0).into()),
        stone: materials.get(Thing::Stone),
        coal: materials.get(Thing::Coal),
        gold: materials.get(Thing::Gold),
        iron: materials.get(Thing::Iron),
    });
}

fn update_boulder_config(
    config: Res<BoulderConfig>,
    boulders: Query<(Entity, &Boulder, &Transform)>,
    mut spawn: BoulderSpawn,
    mut cmds: Commands,
) {
    if !config.is_changed() {
        return;
    }

    for (entity, boulder, transform) in boulders.iter() {
        let transform = Transform::from_translation(transform.translation);
        spawn.spawn(boulder.clone(), transform);
        cmds.entity(entity).despawn_recursive();
    }
}
