use bevy::{
    ecs::{entity::EntityMap, system::SystemParam},
    input::{mouse::MouseWheel, system::exit_on_esc_system},
    prelude::shape::Capsule,
    prelude::*,
};
use bevy_egui::egui::{self, FontDefinitions, FontFamily};
use bevy_egui::*;
pub use bevy_mod_picking::*;

use crate::entities::*;
use crate::{entities::tree::MarkCutTree, systems::Stack};
use crate::{
    entities::{ritual_site::RitualSite, sign::Sign, tree::Tree},
    systems::MeshModifiers,
};

use super::{
    interact_with_focus::InteractWithFocusEvent, things::Thing, AugmentSpawn, CameraTracking,
    Destructor, Focus,
};

pub struct UserInputPlugin;

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(PickingPlugin)
            .add_plugin(InteractablePickingPlugin)
            .add_plugin(HighlightablePickingPlugin)
            .add_plugin(EguiPlugin)
            .add_system(exit_on_esc_system)
            .add_system(make_pickable)
            .add_system(example_ui)
            .add_system(click_imp)
            .add_system(update_pedestals)
            .add_system(player_movement)
            .add_system(update_player)
            .add_system(camera_zoom_with_mousewheel)
            .add_system(update_tree_rings)
            .insert_resource(UiState::default())
            .init_resource::<DebugConfig>();
    }
}

#[derive(Default)]
pub struct DebugConfig {
    pub imp_walk_destination: bool,
    pub spawn_chains_belt_def_duration: f32,
    pub tree_capsule_mesh: Option<Handle<Mesh>>,
    pub tree_capsule: shape::Capsule,
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
    Sign,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            build_tool_state: BuildToolState {
                build: Build::Conveyor,
                boulder_kind: Thing::Stone,
                start_line: None,
            },
        }
    }
}

impl Default for UiMode {
    fn default() -> Self {
        Self::BuildTool
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

impl BackAndForth for Thing {
    fn elems(&self) -> &[Self] {
        &BOULDER_MATERIALS
    }
}

const BUILDS: [Build; 4] = [Build::Boulder, Build::Conveyor, Build::Imp, Build::Sign];
const BOULDER_MATERIALS: [Thing; 4] = [Thing::Stone, Thing::Coal, Thing::Iron, Thing::Gold];

#[derive(Debug)]
struct BuildToolState {
    build: Build,
    boulder_kind: Thing,
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
    mut sign: sign::Spawn,
    mut destructor: Destructor,
    mut interact_with_focus: EventWriter<InteractWithFocusEvent>,
) {
    let (mage_entity, mage_transform, mut mage, focus) = mage.single_mut();
    let in_front = mage_transform.translation + mage_transform.rotation.mul_vec3(0.5 * Vec3::Z);

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
                        let ghost =
                            conveyor.spawn_dynamic_ghost(mage_transform.translation, mage_entity);
                        state.build_tool_state.start_line =
                            Some((ghost, mage_transform.translation));
                        state.mode = UiMode::BuildConveyorTool;
                    }
                }
                Build::Imp => {
                    if input.just_pressed(KeyCode::E) {
                        imp.spawn(Imp::new(), mage_transform.clone());
                    }
                }
                Build::Sign => {
                    if input.just_pressed(KeyCode::E) {
                        sign.spawn(None, in_front);
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
                if let Some((ghost, _from)) = state.build_tool_state.start_line {
                    conveyor.manifest_ghost(ghost);
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
            transform.rotation = transform.rotation.lerp(
                Quat::from_rotation_y(control.x.atan2(control.z)),
                (10.0 * dt).min(1.0),
            );
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

// #[derive(Default)]
// struct UiState {
//     thing: Option<Thing>,
// }

fn example_ui(
    state: Res<UiState>,
    egui_ctx: Res<EguiContext>,
    details: Details,
    boulder_config: ResMut<BoulderConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut debug_config: ResMut<DebugConfig>,
) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "Nanum".to_owned(),
        std::borrow::Cow::Borrowed(include_bytes!("../../assets/NanumBrushScript-Regular.ttf")),
    );
    fonts.fonts_for_family.insert(
        FontFamily::Proportional,
        vec![
            "Ubuntu-Light".to_owned(),
            "NotoEmoji-Regular".to_owned(),
            "emoji-icon-font".to_owned(),
            "Nanum".to_owned(),
        ],
    );
    egui_ctx.ctx().set_fonts(fonts);

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
                    Build::Sign => {
                        ui.label(format!("{:?}. (E) spawn (Q) cancel", Build::Sign));
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
            ui.add(
                egui::Slider::new(&mut debug_config.spawn_chains_belt_def_duration, 0.0..=10.0)
                    .integer()
                    .text("spawn chains belt defs duration"),
            );

            //

            ui.heading("Debug tree capsule");

            let mut c = debug_config.tree_capsule;
            ui.add(egui::Slider::new(&mut c.depth, 0.0..=1.0).text("depth"));
            ui.add(egui::Slider::new(&mut c.radius, 0.0..=1.0).text("radius"));
            ui.add(egui::Slider::new(&mut c.latitudes, 2..=20).text("latitudes"));
            ui.add(egui::Slider::new(&mut c.longitudes, 2..=20).text("longitudes"));
            ui.add(egui::Slider::new(&mut c.rings, 1..=10).text("rings"));

            if !capsule_eq(&c, &debug_config.tree_capsule) {
                debug_config.tree_capsule = c;

                if let Some(handle) = debug_config.tree_capsule_mesh.clone() {
                    let handle =
                        meshes.set(handle, Mesh::from(debug_config.tree_capsule).displace(0.05));
                    debug_config.tree_capsule_mesh = Some(handle);
                }
            }

            //

            ui.add_space(8.0);
            boulder_config_ui(ui, boulder_config);
        });

    if let Some(pos) = details.focus_screen_pos() {
        let height = egui_ctx.ctx().available_rect().height();
        let pos = (pos.x, height - pos.y);

        egui::Area::new("interaction_hint")
            .fixed_pos(pos)
            .show(egui_ctx.ctx(), |ui| {
                details.interaction_hint(ui);
            });
    } else {
    }
}

fn capsule_eq(a: &Capsule, b: &Capsule) -> bool {
    a.depth == b.depth
        && a.radius == b.radius
        && a.latitudes == b.latitudes
        && a.longitudes == b.longitudes
        && a.rings == b.rings
    // IGNORE a.uv_profile == b.uv_profile
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
    belt: Query<'w, 's, &'static ConveyorBelt>,
    boulder: Query<'w, 's, &'static Boulder>,
    pile: Query<'w, 's, &'static Pile>,
    tree: Query<'w, 's, &'static tree::Tree>,
    ritual_site: Query<'w, 's, &'static RitualSite>,
    sign: Query<'w, 's, &'static Sign>,
}

impl<'w, 's> Details<'w, 's> {
    fn focus_screen_pos(&self) -> Option<Vec2> {
        self.focus
            .get_single()
            .ok()
            .and_then(|focus| focus.screen_pos)
    }

    fn interaction_hint(&self, ui: &mut egui::Ui) {
        if let Some(entity) = self.focus.get_single().ok().and_then(|focus| focus.entity) {
            if let Ok(boulder) = self.boulder.get(entity) {
                ui.label(format!("Boulder of {:?}", boulder.material));
                if boulder.marked_for_digging {
                    ui.label("Ⓔ stop dig");
                } else {
                    ui.label("Ⓔ let dig");
                }
            }

            if let Ok(pile) = self.pile.get(entity) {
                ui.label(format!("Pile of {:.1} {:?}", pile.amount, pile.load));
                ui.label("Ⓔ take one");
            }

            if let Ok(belt) = self.belt.get(entity) {
                let mage = self.mage.single();
                ui.label(format!("Conveyor for {:?}", belt.marked_for_thing));
                match (belt.marked_for_thing, mage.peek_first()) {
                    (Some(_), None) => {
                        ui.label("(E) unmark");
                    }
                    (_, Some(thing)) => {
                        ui.label(format!("(E) mark with {:?}", thing));
                    }
                    _ => {}
                }
            }

            if let Ok(tree) = self.tree.get(entity) {
                ui.label("Tree");
                if tree.mark_cut_tree {
                    ui.label("Ⓔ stop cutting down");
                } else {
                    ui.label("Ⓔ let cut down");
                }
            }

            if let Ok(site) = self.ritual_site.get(entity) {
                let needs: String = site
                    .needs
                    .iter()
                    .map(|need| format!("{}/{} {:?} ", need.available, need.needed, need.what))
                    .collect();
                ui.label("Ritual site");
                ui.label(format!("{}", needs));
            }

            if let Ok(sign) = self.sign.get(entity) {
                ui.label(format!("Sign of {:?}", sign.thing));
                ui.label("(E) put item");
            }
        }
    }

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
    }
}

fn update_pedestals(
    boulders: Query<(Entity, &Boulder), Changed<Boulder>>,
    mut augment: AugmentSpawn,
) {
    for (boulder_entity, boulder) in boulders.iter() {
        augment.with_pedestal(boulder_entity, boulder.marked_for_digging);
    }
}

fn update_tree_rings(
    added_trees: Query<(Entity, &Tree, &Transform), Added<MarkCutTree>>,
    removed_trees: RemovedComponents<MarkCutTree>,
    mut tree_ring_map: Local<EntityMap>,
    mut augment: AugmentSpawn,
    mut cmds: Commands,
) {
    for (a_tree, tree, t_tree) in added_trees.iter() {
        let pos = Vec3::new(t_tree.translation.x, 0.02, t_tree.translation.z);
        let a_ring = augment.spawn_disk(pos, tree.tree_radius).id();
        tree_ring_map.insert(a_tree, a_ring);
    }

    for a_tree in removed_trees.iter() {
        if let Ok(a_ring) = tree_ring_map.get(a_tree) {
            tree_ring_map.remove(a_tree);
            cmds.entity(a_ring).despawn_recursive();
        }
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
