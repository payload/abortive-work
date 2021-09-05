use std::f32::consts::TAU;

use bevy::{
    ecs::system::SystemParam,
    input::{keyboard::KeyboardInput, system::exit_on_esc_system, ElementState},
    math::vec3,
    prelude::*,
};
use bevy_egui::*;
pub use bevy_mod_picking::*;

use crate::entities::*;

use super::{BuildingTool, BuildingToolPlugin, Buildings, Store, Thing};

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
            .add_system(click_imp)
            .add_system(click_smithery)
            .add_system(player_movement)
            .insert_resource(UiState::default());
    }
}

fn player_movement(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Mage>>,
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

pub struct ImpSpawnPoint;

fn spawn_imp_on_key(
    mut imp: ImpSpawn,
    mut keyboard_input_events: EventReader<KeyboardInput>,
    spawn_point: Query<&Transform, With<ImpSpawnPoint>>,
) {
    let spawn_point = spawn_point
        .single()
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO);

    for event in keyboard_input_events.iter() {
        if let Some(key_code) = event.key_code {
            if event.state == ElementState::Pressed && key_code == KeyCode::I {
                let a = TAU * fastrand::f32();
                let vec = vec3(a.cos(), 0.0, a.sin());
                imp.spawn(Imp::new(), Transform::from_translation(vec + spawn_point));
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
    //
    mage: Query<Entity, With<Mage>>,
    mut imps: Query<(Entity, &Imp, &mut ImpCommands)>,
) {
    for (parent, interaction) in models.iter() {
        if let Interaction::Clicked = interaction {
            let boulder_entity = **parent;
            let boulder = boulders.get_mut(boulder_entity);
            let mage = mage.single();
            if let (Ok(_), Ok(mage)) = (boulder, mage) {
                for (_, _, mut imp_cmds) in imps.iter_mut().filter(|(_, imp, _)| {
                    if let Some(e) = imp.want_to_follow {
                        e == mage
                    } else {
                        false
                    }
                }) {
                    imp_cmds.commands.push(ImpCommand::Dig(boulder_entity));
                }
            }
        }
    }
}

fn click_imp(
    models: Query<(&Parent, &Interaction), (Changed<Interaction>, With<ImpModel>)>,
    mut imp: Query<&mut Imp>,
    mage: Query<Entity, With<Mage>>,
) {
    for (parent, interaction) in models.iter() {
        if let Interaction::Clicked = interaction {
            if let Ok(mut imp) = imp.get_mut(**parent) {
                if let Ok(mage) = mage.single() {
                    imp.maybe_follow(mage);
                }
            }
        }
    }
}

fn click_smithery(
    models: Query<(&Parent, &Interaction), (Changed<Interaction>, With<SmitheryModel>)>,
    mut store: Query<&mut Store>,
    mut mage: Query<&mut Mage>,
) {
    for (parent, interaction) in models.iter() {
        if let Interaction::Clicked = interaction {
            if let Ok(store) = store.get_mut(**parent) {
                if let Ok(mut mage) = mage.single_mut() {
                    if let Some(stack) = store.first_output_stack() {
                        if let Some(thing) = stack.thing {
                            if stack.amount > 0.0 {
                                mage.put_into_inventory(thing, stack.amount.min(1.0));
                            }
                        }
                    }
                }
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

fn example_ui(mut state: ResMut<UiState>, egui_ctx: Res<EguiContext>, details: Details) {
    let mut thing_copy = state.thing;
    let thing = &mut thing_copy;

    egui::Window::new("Thing")
        .scroll(true)
        .default_width(200.0)
        .default_pos((0.0, 0.0))
        .show(egui_ctx.ctx(), |ui| {
            ui.selectable_value(thing, Some(Thing::Stone), "Stone");
            ui.selectable_value(thing, Some(Thing::Coal), "Coal");
            ui.selectable_value(thing, Some(Thing::Iron), "Iron");
            ui.selectable_value(thing, Some(Thing::Gold), "Gold");
            ui.selectable_value(thing, Some(Thing::Tool), "Tool");

            details.add_to_ui(ui);
        });

    state.thing = thing_copy;
}

#[derive(SystemParam)]
pub struct Details<'w, 's> {
    models: Query<'w, 's, (&'static Parent, &'static Selection), With<ImpModel>>,
    imps: Query<'w, 's, &'static Imp>,
    mage: Query<'w, 's, &'static Mage>,
}

impl<'w, 's> Details<'w, 's> {
    fn add_to_ui(&self, ui: &mut egui::Ui) {
        for imp in self
            .models
            .iter()
            .filter(|(_, selection)| selection.selected())
            .filter_map(|(parent, _)| self.imps.get(**parent).ok())
        {
            let desc = format!(
                "{name} {does_something}{and_carries}.",
                name = "imp",
                does_something = match imp.behavior {
                    ImpBehavior::Idle => "does nothing",
                    ImpBehavior::Dig => "diggs",
                    ImpBehavior::Store => "stores",
                    ImpBehavior::Follow(_) => "follows",
                },
                and_carries = match imp.load {
                    Some(load) => format!(" and carries {:.1} {:?}", imp.load_amount, load),
                    None => String::new(),
                }
            );

            ui.label(desc);
        }

        for mage in self.mage.single() {
            let inventory: String = mage
                .inventory
                .iter()
                .filter_map(|stack| {
                    stack
                        .thing
                        .map(|thing| format!(" {:.1} {:?}", stack.amount, thing))
                })
                .collect();

            let desc = format!("mage has{}", inventory);

            ui.label(desc);
        }
    }
}
