use std::f32::consts::TAU;

use bevy::{
    input::{keyboard::KeyboardInput, system::exit_on_esc_system, ElementState},
    math::vec3,
    prelude::*,
    render::camera::Camera,
};
use bevy_egui::*;
pub use bevy_mod_picking::*;

use crate::entities::*;

use super::{BuildingTool, BuildingToolPlugin, Buildings, Thing};

pub struct UserInputPlugin;

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(PickingPlugin)
            .add_plugin(InteractablePickingPlugin)
            .add_plugin(HighlightablePickingPlugin)
            .add_plugin(BuildingToolPlugin)
            .add_plugin(EguiPlugin)
            .add_system(exit_on_esc_system)
            .add_system(spawn_imp_on_key)
            .add_system(make_pickable)
            .add_system(click_boulder)
            .add_system(interact_ground)
            .add_system(example_ui)
            .add_system(move_camera)
            .insert_resource(UiState::default());
    }
}

fn move_camera(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    let speed = if input.pressed(KeyCode::LShift) {
        8.0
    } else {
        3.0
    };

    let dt = time.delta_seconds();
    let mut control = Vec3::ZERO;

    if input.pressed(KeyCode::W) {
        control.z += 1.0;
    }
    if input.pressed(KeyCode::S) {
        control.z -= 1.0;
    }
    if input.pressed(KeyCode::A) {
        control.x += 1.0;
    }
    if input.pressed(KeyCode::D) {
        control.x -= 1.0;
    }

    control = control.normalize_or_zero();

    if let Ok(mut transform) = query.single_mut() {
        if control != Vec3::ZERO {
            transform.translation += control.normalize_or_zero() * speed * dt;
        }
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
    camera: Query<&PickingCamera>,
    models: Query<(Entity, &Parent, &Interaction), With<GroundModel>>,
    interaction_changed: Query<&Interaction, Changed<Interaction>>,
    mut building_tool: ResMut<BuildingTool>,
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut build_mode: Local<bool>,
) {
    for event in keyboard_input_events.iter() {
        if let Some(key_code) = event.key_code {
            if event.state == ElementState::Pressed && key_code == KeyCode::B {
                let building = building_tool.building.unwrap_or(Buildings::StoneBoulder);

                if *build_mode {
                    building_tool.building = Some(building.next());
                } else {
                    *build_mode = true;
                    building_tool.building = Some(building);
                }
            }
            if event.state == ElementState::Pressed && key_code == KeyCode::Return {
                if *build_mode {
                    *build_mode = false;
                    building_tool.ghost_visible = false;
                }
            }
        }
    }

    if !*build_mode {
        return;
    }

    if let Ok(camera) = camera.single() {
        for (ground_model_entity, _parent, interaction) in models.iter() {
            let changed = interaction_changed.get(ground_model_entity).is_ok();

            let round_pos = if let Some((_, intersection)) = camera.intersect_top() {
                intersection.position().round()
            } else {
                Vec3::ZERO
            };

            match interaction {
                Interaction::Clicked if changed => {
                    building_tool.build = true;
                    building_tool.ghost_visible = false;
                }
                Interaction::Hovered => {
                    building_tool.ghost_visible = true;
                    building_tool.placement.translation = round_pos;
                }
                Interaction::None if changed => {
                    building_tool.ghost_visible = false;
                }
                _ => {}
            }
        }
    }
}

#[derive(Default)]
struct UiState {
    thing: Option<Thing>,
}

fn example_ui(mut state: ResMut<UiState>, egui_ctx: Res<EguiContext>) {
    let mut thing_copy = state.thing;
    let thing = &mut thing_copy;

    egui::Window::new("Thing")
        .scroll(true)
        .default_width(100.0)
        .show(egui_ctx.ctx(), |ui| {
            ui.selectable_value(thing, Some(Thing::Stone), "Stone");
            ui.selectable_value(thing, Some(Thing::Coal), "Coal");
            ui.selectable_value(thing, Some(Thing::Iron), "Iron");
            ui.selectable_value(thing, Some(Thing::Gold), "Gold");
            ui.selectable_value(thing, Some(Thing::Tool), "Tool");
        });

    state.thing = thing_copy;
}
