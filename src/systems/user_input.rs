use bevy::{
    ecs::system::SystemParam,
    input::{keyboard::KeyboardInput, mouse::MouseWheel, system::exit_on_esc_system, ElementState},
    prelude::*,
};
use bevy_egui::egui;
use bevy_egui::*;
pub use bevy_mod_picking::*;

use crate::entities::ritual_site::RitualSite;
use crate::entities::*;
use crate::{entities::tree::MarkCutTree, systems::Stack};

use super::{
    interact_with_focus::InteractWithFocusEvent, AugmentSpawn, BuildingTool, BuildingToolPlugin,
    Buildings, CameraTracking, Destructor, Focus, Store,
};

pub struct UserInputPlugin;

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(PickingPlugin)
            .add_plugin(InteractablePickingPlugin)
            .add_plugin(HighlightablePickingPlugin)
            .add_plugin(BuildingToolPlugin)
            .add_plugin(EguiPlugin)
            .add_system(exit_on_esc_system)
            .add_system(make_pickable)
            .add_system(interact_ground)
            .add_system(example_ui)
            .add_system(click_imp)
            .add_system(click_smithery)
            .add_system_to_stage(CoreStage::PostUpdate, update_mage_focus)
            .add_system(update_pedestals)
            .add_system(player_movement)
            .add_system(update_player)
            .add_system(camera_zoom_with_mousewheel)
            .insert_resource(UiState::default())
            .init_resource::<DebugConfig>();
    }
}

#[derive(Default)]
pub struct DebugConfig {
    pub imp_walk_destination: bool,
}

#[derive(Debug)]
struct UiState {
    mode: UiMode,
    build_tool_state: BuildToolState,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum UiMode {
    None,
    BuildTool,
    BuildConveyorTool,
    DestructTool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Build {
    Boulder,
    Conveyor,
    Imp,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            build_tool_state: BuildToolState {
                build: Build::Boulder,
                boulder_kind: BoulderMaterial::Stone,
                start_line: None,
            },
        }
    }
}

impl Default for UiMode {
    fn default() -> Self {
        Self::None
    }
}

impl Default for Build {
    fn default() -> Self {
        Build::Boulder
    }
}

trait BackAndForth: Sized + Eq + Copy + std::fmt::Debug {
    fn elems(&self) -> &[Self];

    fn prev(&self) -> Self {
        let elems = self.elems();
        let pos = elems.iter().position(|e| e == self).unwrap_or(0);
        if pos == 0 {
            *elems.last().unwrap()
        } else {
            *elems.get(pos - 1).unwrap()
        }
    }

    fn next(&self) -> Self {
        let elems = self.elems();
        *elems
            .iter()
            .position(|e| e == self)
            .map(|pos| {
                if pos == elems.len() - 1 {
                    elems.first()
                } else {
                    elems.get(pos + 1)
                }
            })
            .flatten()
            .unwrap_or(self)
    }
}

impl BackAndForth for Build {
    fn elems(&self) -> &[Self] {
        &BUILDS
    }
}

impl BackAndForth for BoulderMaterial {
    fn elems(&self) -> &[Self] {
        &BOULDER_MATERIALS
    }
}

const BUILDS: [Build; 3] = [Build::Boulder, Build::Conveyor, Build::Imp];
const BOULDER_MATERIALS: [BoulderMaterial; 4] = [
    BoulderMaterial::Stone,
    BoulderMaterial::Coal,
    BoulderMaterial::Iron,
    BoulderMaterial::Gold,
];

#[derive(Debug)]
struct BuildToolState {
    build: Build,
    boulder_kind: BoulderMaterial,
    start_line: Option<(Entity, Vec3)>,
}

// J switches into build tool mode
// in build tool mode
//   JL switches build tool back and forth
//   IK switches inside the build tool (material for example)
//   E accepts or continues
//   Q cancels

// TODO follow this description for implementation

fn update_player(
    input: Res<Input<KeyCode>>,
    mut conveyor: ConveyorSpawn,
    mut state: ResMut<UiState>,
    mut mage: Query<(Entity, &Transform, &mut Mage, &Focus), With<Mage>>,
    mut cmds: Commands,
    //
    mut imp: ImpSpawn,
    mut boulder: BoulderSpawn,
    mut destructor: Destructor,
    mut interact_with_focus: EventWriter<InteractWithFocusEvent>,
) {
    let (mage_entity, mage_transform, mut mage, focus) = mage.single_mut();

    match state.mode {
        UiMode::None => {
            if input.just_pressed(KeyCode::J) {
                state.mode = UiMode::BuildTool;
            }
            if input.just_pressed(KeyCode::L) {
                state.mode = UiMode::DestructTool;
            }
            if input.just_pressed(KeyCode::E) {
                interact_with_focus.send(InteractWithFocusEvent);
            }
            if input.just_pressed(KeyCode::O) {
                mage.try_drop_item(mage_transform, &mut cmds);
            }
        }
        UiMode::DestructTool => {
            if input.just_pressed(KeyCode::E) {
                destructor.destruct_some(focus.entity);
            }
            if input.just_pressed(KeyCode::Q) {
                state.build_tool_state.start_line = None;
                state.mode = UiMode::None;
            }
        }
        UiMode::BuildTool => {
            if input.just_pressed(KeyCode::J) {
                state.build_tool_state.build = state.build_tool_state.build.prev();
            }
            if input.just_pressed(KeyCode::L) {
                state.build_tool_state.build = state.build_tool_state.build.next();
            }
            if input.just_pressed(KeyCode::Q) {
                state.build_tool_state.start_line = None;
                state.mode = UiMode::None;
            }

            match state.build_tool_state.build {
                Build::Boulder => {
                    if input.just_pressed(KeyCode::I) {
                        state.build_tool_state.boulder_kind =
                            state.build_tool_state.boulder_kind.prev();
                    }
                    if input.just_pressed(KeyCode::K) {
                        state.build_tool_state.boulder_kind =
                            state.build_tool_state.boulder_kind.next();
                    }
                    if input.just_pressed(KeyCode::E) {
                        boulder.spawn(
                            Boulder::new(state.build_tool_state.boulder_kind),
                            mage_transform.clone(),
                        );
                    }
                }
                Build::Conveyor => {
                    if input.just_pressed(KeyCode::E) {
                        let line = conveyor.ghostline_from_point_to_entity(
                            mage_transform.translation,
                            mage_entity,
                        );
                        state.build_tool_state.start_line =
                            Some((line, mage_transform.translation));
                        state.mode = UiMode::BuildConveyorTool;
                    }
                }
                Build::Imp => {
                    if input.just_pressed(KeyCode::E) {
                        imp.spawn(Imp::new(), mage_transform.clone());
                    }
                }
            }
        }
        UiMode::BuildConveyorTool => {
            if input.just_pressed(KeyCode::Q) {
                if let Some((line, _)) = state.build_tool_state.start_line {
                    cmds.entity(line).despawn_recursive();
                    state.build_tool_state.start_line = None;
                }
                state.mode = UiMode::BuildTool;
            } else if input.just_pressed(KeyCode::E) {
                if let Some((line, from)) = state.build_tool_state.start_line {
                    cmds.entity(line).despawn_recursive();
                    conveyor.spawn_line(from, mage_transform.translation);
                    state.build_tool_state.start_line = None;
                }
                state.mode = UiMode::BuildTool;
            }
        }
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

    if let Ok(mut transform) = query.get_single_mut() {
        if control != Vec3::ZERO {
            transform.translation += control.normalize_or_zero() * speed * dt;
            transform.rotation = Quat::from_rotation_y(control.x.atan2(control.z));
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

fn click_imp(
    models: Query<(&Parent, &Interaction), (Changed<Interaction>, With<ImpModel>)>,
    mut imp: Query<&mut Imp>,
    mage: Query<Entity, With<Mage>>,
) {
    for (parent, interaction) in models.iter() {
        if let Interaction::Clicked = interaction {
            if let Ok(mut imp) = imp.get_mut(**parent) {
                if let Ok(mage) = mage.get_single() {
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
                if let Ok(mut mage) = mage.get_single_mut() {
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

    if let Ok(camera) = camera.get_single() {
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

// #[derive(Default)]
// struct UiState {
//     thing: Option<Thing>,
// }

fn example_ui(
    state: Res<UiState>,
    egui_ctx: Res<EguiContext>,
    details: Details,
    boulder_config: ResMut<BoulderConfig>,
    mut debug_config: ResMut<DebugConfig>,
) {
    egui::Window::new("abortive work")
        .default_pos((0.0, 0.0))
        .show(egui_ctx.ctx(), |ui| {
            ui.heading(format!("{:?}", state.mode));

            match state.mode {
                UiMode::None => {
                    ui.label("(J) build tool (L) destruct tool");
                }
                UiMode::BuildTool => match state.build_tool_state.build {
                    Build::Boulder => {
                        ui.label(format!(
                            "{:?} {:?}. (E) build (Q) cancel",
                            state.build_tool_state.boulder_kind,
                            Build::Boulder
                        ));
                    }
                    Build::Conveyor => {
                        ui.label(format!("{:?}. (E) start belt (Q) cancel", Build::Conveyor));
                    }
                    Build::Imp => {
                        ui.label(format!("{:?}. (E) spawn (Q) cancel", Build::Imp));
                    }
                },
                UiMode::DestructTool => {
                    ui.label("(E) destruct (Q) cancel");
                }
                UiMode::BuildConveyorTool => {
                    ui.label("(E) build (Q) cancel");
                }
            }

            ui.add_space(8.0);
            details.add_to_ui(ui);

            ui.add_space(8.0);
            ui.heading("Debug config");
            ui.checkbox(
                &mut debug_config.imp_walk_destination,
                "imp walk destination",
            );

            ui.add_space(8.0);
            boulder_config_ui(ui, boulder_config);
        });
}

fn boulder_config_ui(ui: &mut egui::Ui, mut boulder_config: ResMut<BoulderConfig>) {
    ui.heading("Boulder config");
    let mut value = boulder_config.max_angle_deviation.to_degrees();
    ui.add(
        egui::Slider::new(&mut value, 0.0..=45.0)
            .integer()
            .text("Boulder max angle deviation"),
    );
    let value = value.to_radians();
    if boulder_config.max_angle_deviation != value {
        boulder_config.max_angle_deviation = value;
    }
}

#[derive(SystemParam)]
pub struct Details<'w, 's> {
    mage: Query<'w, 's, &'static Mage>,
    focus: Query<'w, 's, &'static Focus>,
    conveyor: Query<'w, 's, &'static ConveyorBelt>,
    boulder: Query<'w, 's, &'static Boulder>,
    pile: Query<'w, 's, &'static Pile>,
    ritual_site: Query<'w, 's, &'static RitualSite>,
}

impl<'w, 's> Details<'w, 's> {
    fn add_to_ui(&self, ui: &mut egui::Ui) {
        let display_stack_thing = |s: &Stack| s.thing.map(|t| format!(" {:.1} {:?}", s.amount, t));

        ui.heading("Mage");
        ui.label(
            self.mage
                .get_single()
                .map(|mage| {
                    std::iter::once("Carries".to_string())
                        .chain(mage.inventory.iter().filter_map(display_stack_thing))
                        .collect::<String>()
                })
                .unwrap_or_default(),
        );

        ui.heading("Ritual");
        ui.label(if let Ok(site) = self.ritual_site.get_single() {
            site.needs
                .iter()
                .map(|need| format!("{}/{} {:?} ", need.available, need.needed, need.what))
                .collect()
        } else {
            String::new()
        });

        ui.heading("Focus");
        ui.label(
            if let Some(entity) = self.focus.get_single().ok().and_then(|focus| focus.entity) {
                if let Ok(conveyor) = self.conveyor.get(entity) {
                    let mage = self.mage.single();
                    format!(
                        "Conveyor for {:?}. {}",
                        conveyor.marked_for_thing,
                        match (conveyor.marked_for_thing, mage.peek_first()) {
                            (Some(_), None) => "(E) unmark".into(),
                            (_, Some(thing)) => format!("(E) mark with {:?}", thing),
                            _ => String::new(),
                        }
                    )
                } else if let Ok(boulder) = self.boulder.get(entity) {
                    format!(
                        "Boulder of {:?}. (E) {}",
                        boulder.material,
                        match boulder.marked_for_digging {
                            true => "stop dig",
                            false => "let dig",
                        }
                    )
                } else if let Ok(pile) = self.pile.get(entity) {
                    format!("Pile of {:.1} {:?}. (E) take one", pile.amount, pile.load)
                } else if let Ok(site) = self.ritual_site.get(entity) {
                    let needs: String = site
                        .needs
                        .iter()
                        .map(|need| format!("{}/{} {:?} ", need.available, need.needed, need.what))
                        .collect();
                    format!("Ritual site with following thing requested:\n{}", needs)
                } else {
                    String::new()
                }
            } else {
                String::new()
            },
        );
    }
}

fn update_mage_focus(
    query: Query<&Focus, (With<Mage>, Changed<Focus>)>,
    entities: Query<Entity>,
    mut augment: AugmentSpawn,
) {
    for focus in query.iter() {
        if let Some(entity) = focus.entity {
            if entities.get(entity).is_ok() {
                augment.add_coin(entity);
            }
        }

        if let Some(entity) = focus.before {
            if entities.get(entity).is_ok() {
                augment.remove_coin(entity);
            }
        }
    }
}

fn update_pedestals(
    boulders: Query<(Entity, &Boulder), Changed<Boulder>>,
    added_trees: Query<Entity, Added<MarkCutTree>>,
    removed_trees: RemovedComponents<MarkCutTree>,
    mut augment: AugmentSpawn,
) {
    for (boulder_entity, boulder) in boulders.iter() {
        augment.with_pedestal(boulder_entity, boulder.marked_for_digging);
    }

    for entity in added_trees.iter() {
        augment.with_pedestal(entity, true);
    }

    for entity in removed_trees.iter() {
        augment.with_pedestal(entity, false);
    }
}

fn camera_zoom_with_mousewheel(
    mut events: EventReader<MouseWheel>,
    mut tracking: Query<&mut CameraTracking>,
) {
    let mut y = 0.0;

    for wheel in events.iter() {
        y += wheel.y;
    }

    if y != 0.0 {
        tracking.single_mut().offset.y += y;
    }
}
