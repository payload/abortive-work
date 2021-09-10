use std::f32::consts::PI;

use bevy::{
    ecs::system::{EntityCommands, SystemParam},
    prelude::*,
};

use crate::systems::{cone, Destructable, FocusObject, Thing};

use super::{MageInteractable, NotGround};

pub struct Pile {
    pub load: Thing,
    pub amount: f32,
}

impl Pile {
    pub fn new(load: Thing, amount: f32) -> Self {
        Self { load, amount }
    }
}

pub struct PilePlugin;

impl Plugin for PilePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets)
            .add_system(store_into_pile)
            .add_system(update_pile)
            .add_system_to_stage(CoreStage::Last, despawn_empty_piles);
    }
}

#[derive(SystemParam)]
pub struct PileSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, PileAssets>,
}

impl<'w, 's> PileSpawn<'w, 's> {
    pub fn spawn<'a>(&'a mut self, pile: Pile, transform: Transform) -> EntityCommands<'w, 's, 'a> {
        let model = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: self.assets.transform.clone(),
                material: self.assets.material.clone(),
                mesh: self.assets.mesh.clone(),
                ..Default::default()
            })
            .insert(NotGround)
            .id();

        let mut entity_cmds = self.cmds.spawn_bundle((
            pile,
            transform,
            GlobalTransform::identity(),
            MageInteractable::default(),
            Destructable,
            FocusObject,
        ));
        entity_cmds.push_children(&[model]);
        entity_cmds
    }
}

#[derive(Clone)]
pub struct PileAssets {
    pub transform: Transform,
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
}

fn load_assets(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.insert_resource(PileAssets {
        transform: Transform::from_xyz(0.0, 0.001, 0.0),
        material: materials.add(StandardMaterial {
            base_color: Color::DARK_GRAY,
            metallic: 0.0,
            roughness: 0.0,
            reflectance: 0.0,
            ..Default::default()
        }),
        mesh: meshes.add(cone(0.1, 0.2, 8)),
    });
}

pub struct StoreIntoPile {
    pub load: Thing,
    pub amount: f32,
    pub pile: Option<Entity>,
}

fn store_into_pile(
    stores: Query<(Entity, &StoreIntoPile, Option<&Transform>)>,
    mut piles: Query<(&mut Pile, &Transform)>,
    mut cmds: Commands,
    mut pile_spawn: PileSpawn,
) {
    for (store_entity, store, store_transform) in stores.iter() {
        cmds.entity(store_entity).despawn_recursive();

        if let Some(store_pile) = store.pile {
            if let Ok((mut pile, _)) = piles.get_mut(store_pile) {
                if pile.amount == 0.0 {
                    pile.load = store.load;
                    pile.amount = store.amount;
                } else if pile.amount > 0.0 && pile.load == store.load {
                    pile.amount += store.amount;
                } else {
                    // TODO create another pile
                }
            }
        } else if let Some(store_transform) = store_transform {
            let mut stored = false;

            'find_pile: for (mut pile, pile_transform) in piles.iter_mut() {
                let dist = pile_transform
                    .translation
                    .distance_squared(store_transform.translation);
                if dist < 1.0 {
                    if pile.amount == 0.0 {
                        pile.load = store.load;
                        pile.amount = store.amount;
                        stored = true;
                        break 'find_pile;
                    } else if pile.amount > 0.0 && pile.load == store.load {
                        pile.amount += store.amount;
                        stored = true;
                        break 'find_pile;
                    }
                }
            }

            if !stored {
                pile_spawn.spawn(Pile::new(store.load, store.amount), store_transform.clone());
            }
        }
    }
}

fn update_pile(mut piles: Query<(&Pile, &mut Transform), Changed<Pile>>) {
    for (pile, mut transform) in piles.iter_mut() {
        // V = pi * r*r * h/3
        // h = r
        let v = pile.amount;
        let h = (12.0 * v / PI).powf(0.3333);
        let scale = Vec3::new(h, h, h);
        if transform.scale != scale {
            transform.scale = scale;
        }
    }
}

fn despawn_empty_piles(piles: Query<(Entity, &Pile)>, mut cmds: Commands) {
    for (entity, pile) in piles.iter() {
        if pile.amount <= 0.0 {
            cmds.entity(entity).despawn_recursive();
        }
    }
}
