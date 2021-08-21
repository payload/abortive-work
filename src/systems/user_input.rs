use std::f32::consts::TAU;

use bevy::{
    input::{keyboard::KeyboardInput, system::exit_on_esc_system, ElementState},
    math::vec3,
    prelude::*,
};
pub use bevy_mod_picking::*;

use crate::entities::*;

pub struct UserInputPlugin;

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(PickingPlugin)
            .add_plugin(InteractablePickingPlugin)
            .add_plugin(HighlightablePickingPlugin)
            .add_startup_system_to_stage(StartupStage::PreStartup, load_ghost_assets)
            .add_system(exit_on_esc_system)
            .add_system(spawn_imp_on_key)
            .add_system(make_pickable)
            .add_system(click_boulder)
            .add_system(interact_ground);
    }
}

fn spawn_imp_on_key(mut imp: ImpSpawn, mut keyboard_input_events: EventReader<KeyboardInput>) {
    for event in keyboard_input_events.iter() {
        if let Some(key_code) = event.key_code {
            if event.state == ElementState::Pressed && key_code == KeyCode::I {
                let a = TAU * fastrand::f32();
                let vec = vec3(a.cos(), 0.0, a.sin());
                imp.spawn(Imp::new(), Transform::from_xyz(vec.x, 0.0, vec.z));
            }
        }
    }
}

fn make_pickable(
    mut cmds: Commands,
    query: Query<Entity, Or<(Added<NotGround>, Added<GroundModel>)>>,
) {
    for entity in query.iter() {
        cmds.entity(entity)
            .insert_bundle((PickableMesh::default(), Interaction::None));
    }
}

fn click_boulder(
    models: Query<(&Parent, &Interaction), (Changed<Interaction>, With<BoulderModel>)>,
    mut boulders: Query<&mut Boulder>,
) {
    for (parent, interaction) in models.iter() {
        if let Interaction::Clicked = interaction {
            if let Ok(mut boulder) = boulders.get_mut(**parent) {
                boulder.marked_for_digging = !boulder.marked_for_digging;
            }
        }
    }
}

fn interact_ground(
    mut cmds: Commands,
    camera: Query<&PickingCamera>,
    models: Query<(Entity, &Parent, &Interaction), With<GroundModel>>,
    interaction_changed: Query<&Interaction, Changed<Interaction>>,
    mut storage: StorageSpawn,
    assets: Res<StorageAssets>,
    mut ghost: Query<(Entity, &mut Transform), With<Ghost>>,
    ghost_assets: Res<GhostAssets>,
) {
    if let Ok(camera) = camera.single() {
        for (ground_model_entity, _parent, interaction) in models.iter() {
            let changed = interaction_changed.get(ground_model_entity).is_ok();

            let round_pos = if let Some((_, intersection)) = camera.intersect_top() {
                intersection.position().round()
            } else {
                Vec3::ZERO
            };
            let round_trans = Transform::from_translation(round_pos);

            match interaction {
                Interaction::Clicked if changed => {
                    //
                    // TODO interact with some building tool resource instead
                    //
                    storage.spawn(Storage, round_trans);
                    if let Ok((ghost_entity, _ghost_transform)) = ghost.single_mut() {
                        cmds.entity(ghost_entity).despawn_recursive();
                    }
                }
                Interaction::Clicked => {}
                Interaction::Hovered if changed => {
                    if let Ok((_ghost_entity, mut ghost_transform)) = ghost.single_mut() {
                        ghost_transform.translation = round_pos;
                    } else {
                        let model = cmds
                            .spawn_bundle(PbrBundle {
                                transform: assets.transform.clone(),
                                mesh: assets.mesh.clone(),
                                material: ghost_assets.material.clone(),
                                ..Default::default()
                            })
                            .id();

                        cmds.spawn_bundle((Ghost, round_trans, GlobalTransform::identity()))
                            .push_children(&[model]);
                    }
                }
                Interaction::Hovered => {
                    if let Ok((_ghost_entity, mut ghost_transform)) = ghost.single_mut() {
                        ghost_transform.translation = round_pos;
                    }
                }
                Interaction::None => {
                    if let Ok((ghost_entity, _ghost_transform)) = ghost.single_mut() {
                        cmds.entity(ghost_entity).despawn_recursive();
                    }
                }
            }
        }
    }
}

struct Ghost;

struct GhostAssets {
    material: Handle<StandardMaterial>,
}

fn load_ghost_assets(mut cmds: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    cmds.insert_resource(GhostAssets {
        material: materials.add(StandardMaterial {
            unlit: true,
            base_color: Color::rgba(1.0, 1.0, 1.0, 0.5),
            ..Default::default()
        }),
    })
}
