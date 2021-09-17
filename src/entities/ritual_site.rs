use crate::systems::*;
use bevy::{ecs::system::SystemParam, prelude::*};

use super::Pile;

#[derive(Default)]
pub struct RitualSite {
    pub needs: Vec<Need>,
    pub radius: f32,
}
pub struct Need {
    pub what: Thing,
    pub needed: u32,
    pub available: u32,
}
pub struct Model;
pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, init_resource)
            .add_system(update);
    }
}

impl RitualSite {
    pub fn new(needs: &[(Thing, u32)]) -> Self {
        Self {
            needs: needs
                .iter()
                .copied()
                .map(|(what, needed)| Need {
                    what,
                    needed,
                    available: 0,
                })
                .collect(),
            radius: 2.0,
            ..Self::default()
        }
    }
}

#[derive(SystemParam)]
pub struct Spawn<'w, 's> {
    cmds: Commands<'w, 's>,
    res: Res<'w, Resource>,
}

impl<'w, 's> Spawn<'w, 's> {
    pub fn spawn(&mut self, component: RitualSite, transform: Transform) {
        let model = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: self.res.transform.clone(),
                material: self.res.material.clone(),
                mesh: self.res.mesh.clone(),
                ..Default::default()
            })
            .insert(Model)
            .id();

        self.cmds
            .spawn_bundle((
                component,
                transform,
                GlobalTransform::identity(),
                Destructable,
                FocusObject,
            ))
            .push_children(&[model]);
    }
}

#[derive(Clone)]
pub struct Resource {
    pub transform: Transform,
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
}

fn init_resource(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: ResMut<AssetServer>,
) {
    cmds.insert_resource(Resource {
        transform: Transform {
            rotation: Quat::IDENTITY,
            translation: Vec3::new(0.0, 0.005, 0.0),
            scale: Vec3::ONE,
        },
        material: materials.add(StandardMaterial {
            base_color_texture: Some(assets.load("nebra4.jpg")),
            unlit: true,
            ..Default::default()
        }),
        mesh: meshes.add(disk(1.0, 24)),
    });
}

fn update(mut sites: Query<(&mut RitualSite, &Transform)>, piles: Query<(&Pile, &Transform)>) {
    let piles_near = |pos, radius_sqr| {
        piles
            .iter()
            .filter(move |(_, t)| t.translation.distance_squared(pos) <= radius_sqr)
    };

    for (mut site, site_transform) in sites.iter_mut() {
        for need in site.needs.iter_mut() {
            need.available = 0;
        }

        for (pile, _pile_transform) in piles_near(site_transform.translation, site.radius) {
            for need in site.needs.iter_mut() {
                if pile.load == need.what {
                    need.available += pile.amount as u32;
                }
            }
        }
    }
}
