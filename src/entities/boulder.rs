use bevy::{ecs::system::SystemParam, prelude::*};

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
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_boulder_assets);
    }
}

#[derive(SystemParam)]
pub struct BoulderSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, BoulderAssets>,
}

impl<'w, 's> BoulderSpawn<'w, 's> {
    pub fn spawn(&mut self, boulder: Boulder, transform: Transform) {
        let model = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: self.assets.transform.clone(),
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
            .spawn_bundle((boulder, transform, GlobalTransform::identity()))
            .push_children(&[model]);
    }
}

fn load_boulder_assets(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.insert_resource(BoulderAssets {
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        stone: materials.add(material(Color::DARK_GRAY)),
        coal: materials.add(material(Color::BLACK)),
        gold: materials.add(material(Color::GOLD)),
        iron: materials.add(material(Color::ORANGE_RED)),
        mesh: meshes.add(shape::Box::new(0.8, 1.0, 0.8).into()),
    });

    fn material(color: Color) -> StandardMaterial {
        StandardMaterial {
            base_color: color,
            ..Default::default()
        }
    }
}
