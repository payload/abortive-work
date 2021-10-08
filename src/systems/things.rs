use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Thing {
    Stone,
    Coal,
    Iron,
    Gold,
    Tool,
    Wood,
}

pub struct ThingMaterials {
    stone: Handle<StandardMaterial>,
    coal: Handle<StandardMaterial>,
    iron: Handle<StandardMaterial>,
    gold: Handle<StandardMaterial>,
    tool: Handle<StandardMaterial>,
    wood: Handle<StandardMaterial>,
}

impl ThingMaterials {
    pub fn get(&self, thing: Thing) -> Handle<StandardMaterial> {
        match thing {
            Thing::Stone => &self.stone,
            Thing::Coal => &self.coal,
            Thing::Iron => &self.iron,
            Thing::Gold => &self.gold,
            Thing::Tool => &self.tool,
            Thing::Wood => &self.wood,
        }
        .clone()
    }
}

pub struct ModPlugin;

impl Plugin for ModPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets);
    }
}

fn load_assets(mut cmds: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    cmds.insert_resource(ThingMaterials {
        stone: materials.add(flat_material(Color::DARK_GRAY)),
        coal: materials.add(flat_material(Color::BLACK)),
        iron: materials.add(flat_material(Color::ORANGE_RED)),
        gold: materials.add(flat_material(Color::GOLD)),
        tool: materials.add(flat_material(Color::YELLOW)),
        wood: materials.add(flat_material(Color::rgb(0.53, 0.36, 0.24))),
    });
}

pub fn flat_material(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        metallic: 0.0,
        reflectance: 0.0,
        roughness: 1.0,
        ..Default::default()
    }
}

pub fn unlit_material(color: Color) -> StandardMaterial {
    StandardMaterial {
        unlit: true,
        ..flat_material(color)
    }
}
