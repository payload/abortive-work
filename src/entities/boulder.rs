use bevy::{ecs::system::SystemParam, prelude::*};

pub struct Boulder {
    pub material: BoulderMaterial,
}

pub enum BoulderMaterial {
    Stone,
    Coal,
    Iron,
    Gold,
}

#[derive(Clone)]
pub struct BoulderAssets {
    transform: Transform,
    stone: Handle<StandardMaterial>,
    coal: Handle<StandardMaterial>,
    iron: Handle<StandardMaterial>,
    gold: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
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
        let pbr_bundle = PbrBundle {
            transform: self.assets.transform.clone(),
            material: match boulder.material {
                BoulderMaterial::Stone => self.assets.stone.clone(),
                BoulderMaterial::Coal => self.assets.coal.clone(),
                BoulderMaterial::Iron => self.assets.iron.clone(),
                BoulderMaterial::Gold => self.assets.gold.clone(),
            },
            mesh: self.assets.mesh.clone(),
            ..Default::default()
        };

        self.cmds
            .spawn_bundle((boulder, transform, GlobalTransform::identity()))
            .with_children(|p| {
                p.spawn_bundle(pbr_bundle);
            });
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
            unlit: true,
            base_color: color,
            ..Default::default()
        }
    }
}
