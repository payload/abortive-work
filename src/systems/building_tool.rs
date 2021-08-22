use bevy::prelude::*;

use crate::entities::*;

pub struct BuildingToolPlugin;

impl Plugin for BuildingToolPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BuildingTool::new())
            .add_startup_system_to_stage(StartupStage::PreStartup, load_ghost_assets)
            .add_startup_system(spawn_ghost)
            .add_system(update_building_tool);
    }
}

pub struct BuildingTool {
    pub building: Option<Buildings>,
    pub placement: Transform,
    pub ghost_visible: bool,
    pub build: bool,
}

#[derive(Clone, Copy)]
pub enum Buildings {
    StoneBoulder,
    CoalBoulder,
    IronBoulder,
    GoldBoulder,
    Imp,
    Storage,
    Smithery,
}

impl BuildingTool {
    fn new() -> Self {
        Self {
            building: None,
            placement: Transform::identity(),
            ghost_visible: false,
            build: false,
        }
    }
}

impl Buildings {
    pub fn next(&self) -> Self {
        use Buildings::*;

        match self {
            StoneBoulder => CoalBoulder,
            CoalBoulder => IronBoulder,
            IronBoulder => GoldBoulder,
            GoldBoulder => Imp,
            Imp => Storage,
            Storage => Smithery,
            Smithery => StoneBoulder,
        }
    }
}

fn update_building_tool(
    mut tool: ResMut<BuildingTool>,
    mut ghost: Query<&mut Transform, With<Ghost>>,
    mut ghost_model: Query<
        (&mut Visible, &mut Transform, &mut Handle<Mesh>),
        (Without<Ghost>, With<GhostModel>),
    >,
    assets: EntityAssets,
    mut spawns: EntitySpawns,
) {
    if !tool.is_changed() {
        return;
    }

    if let Ok((mut model_visible, mut model_transform, mut model_mesh)) = ghost_model.single_mut() {
        let ghost_visible = tool.ghost_visible && tool.building.is_some();

        if model_visible.is_visible != ghost_visible {
            model_visible.is_visible = ghost_visible;
        }

        if ghost_visible {
            if let Ok(mut ghost_transform) = ghost.single_mut() {
                ghost_transform.translation = tool.placement.translation;
            }
        }

        if let Some(building) = &tool.building {
            use Buildings::*;

            let (transform, mesh) = match building {
                StoneBoulder | CoalBoulder | IronBoulder | GoldBoulder => {
                    (assets.boulder.transform, assets.boulder.mesh.clone())
                }
                Imp => (assets.imp.transform, assets.imp.mesh.clone()),
                Storage => (assets.storage.transform, assets.storage.mesh.clone()),
                Smithery => (assets.smithery.transform, assets.smithery.mesh.clone()),
            };

            if *model_mesh != mesh {
                *model_mesh = mesh;
                *model_transform = transform;
            }
        }
    }

    if tool.build {
        tool.build = false;

        if let Some(building) = &tool.building {
            use crate::BoulderMaterial::*;
            let transform = tool.placement;

            match building {
                Buildings::StoneBoulder => spawns.boulder.spawn(Boulder::new(Stone), transform),
                Buildings::CoalBoulder => spawns.boulder.spawn(Boulder::new(Coal), transform),
                Buildings::IronBoulder => spawns.boulder.spawn(Boulder::new(Iron), transform),
                Buildings::GoldBoulder => spawns.boulder.spawn(Boulder::new(Gold), transform),
                Buildings::Imp => spawns.imp.spawn(Imp::new(), transform),
                Buildings::Storage => spawns.storage.spawn(Storage, transform),
                Buildings::Smithery => spawns.smithery.spawn(Smithery, transform),
            }
        }
    }
}

struct Ghost;
struct GhostModel;

struct GhostAssets {
    material: Handle<StandardMaterial>,
}

fn spawn_ghost(mut cmds: Commands, ghost_assets: Res<GhostAssets>) {
    cmds.spawn_bundle((Ghost, Transform::identity(), GlobalTransform::identity()))
        .with_children(|p| {
            p.spawn_bundle(PbrBundle {
                visible: Visible {
                    is_visible: false,
                    is_transparent: true,
                },
                material: ghost_assets.material.clone(),
                ..Default::default()
            })
            .insert(GhostModel);
        });
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
