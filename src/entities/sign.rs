use crate::systems::*;
use bevy::{ecs::system::SystemParam, prelude::*};

use super::{tree::Tree, Boulder, ConveyorBelt};

#[derive(Clone)]
pub struct Sign {
    pub thing: Option<Thing>,
    pub content_model: Entity,
}
pub struct Model;
pub struct ModPlugin;

impl Plugin for ModPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, init_resource)
            .add_system_to_stage(CoreStage::PreUpdate, display_content)
            .add_system(update_sign_influence)
            .add_system(update_influenced_by_signs);
    }
}

#[derive(SystemParam)]
pub struct Spawn<'w, 's> {
    cmds: Commands<'w, 's>,
    res: Res<'w, Resource>,
}

impl<'w, 's> Spawn<'w, 's> {
    pub fn spawn(&mut self, thing: Option<Thing>, pos: Vec3) {
        let content_model = self.cmds.spawn().id();
        let model = self
            .cmds
            .spawn_bundle(PbrBundle {
                transform: self.res.sign_transform.clone(),
                material: self.res.sign_material.clone(),
                mesh: self.res.sign_mesh.clone(),
                ..Default::default()
            })
            .insert(Model)
            .push_children(&[content_model])
            .id();

        self.cmds
            .spawn_bundle((
                Sign {
                    thing,
                    content_model,
                },
                Transform::from_translation(pos),
                GlobalTransform::identity(),
                Destructable,
                FocusObject::new(),
            ))
            .push_children(&[model]);
    }
}

#[derive(Clone)]
pub struct Resource {
    pub sign_transform: Transform,
    pub sign_material: Handle<StandardMaterial>,
    pub sign_mesh: Handle<Mesh>,
    pub item_mesh: Handle<Mesh>,
}

fn init_resource(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.insert_resource(Resource {
        sign_transform: Transform::from_xyz(0.0, 0.5, 0.0),
        sign_material: materials.add(flat_material(Color::AQUAMARINE)),
        sign_mesh: meshes.add(ring(0.175, 0.125, 16)),
        item_mesh: meshes.add(disk(0.125, 16)),
    });
}

fn display_content(
    signs: Query<&Sign, Changed<Sign>>,
    mut visible: Query<&mut Visible>,
    mut cmds: Commands,
    res: Res<Resource>,
    materials: Res<ThingMaterials>,
) {
    for sign in signs.iter() {
        if let Some(thing) = sign.thing {
            cmds.entity(sign.content_model).insert_bundle(PbrBundle {
                mesh: res.item_mesh.clone(),
                material: materials.get(thing),
                ..Default::default()
            });
        } else if let Ok(mut visible) = visible.get_mut(sign.content_model) {
            visible.is_visible = false;
        }
    }
}

#[derive(Default)]
struct Influence {
    signs: Vec<Entity>,
}

fn update_sign_influence(
    signs: Query<(Entity, &Transform), With<Sign>>,
    others: Query<(Entity, &Transform), Or<(With<Boulder>, With<ConveyorBelt>, With<Tree>)>>,
    mut influences: Query<&mut Influence>,
    mut cmds: Commands,
) {
    for mut influence in influences.iter_mut() {
        influence.signs.clear();
    }

    for (sign_entity, sign) in signs.iter() {
        for (other_entity, other) in others.iter() {
            if sign.translation.distance_squared(other.translation) < 4.0 {
                if let Ok(mut influence) = influences.get_mut(other_entity) {
                    influence.signs.push(sign_entity);
                } else {
                    // just insert component, next frame it is there, thats good enough
                    cmds.entity(other_entity).insert(Influence::default());
                }
            }
        }
    }
}

fn update_influenced_by_signs(
    signs: Query<&Sign>,
    mut influenced: Query<(
        &Influence,
        Option<&mut Boulder>,
        Option<&mut ConveyorBelt>,
        Option<&mut Tree>,
    )>,
) {
    for (influence, boulder, belt, tree) in influenced.iter_mut() {
        let signs: Vec<Sign> = influence
            .signs
            .iter()
            .filter_map(|sign| signs.get(*sign).ok())
            .cloned()
            .collect();

        if let Some(mut boulder) = boulder {
            let mut mark = false;

            for sign in signs.iter() {
                if let Some(thing) = sign.thing {
                    if thing == boulder.material {
                        mark = true;
                    }
                }
            }

            if boulder.marked_for_digging != mark {
                boulder.marked_for_digging = mark;
            }
        }

        if let Some(mut belt) = belt {
            let mut thing = None;

            for sign in signs.iter() {
                if sign.thing.is_some() {
                    thing = sign.thing;
                }
            }

            if belt.marked_for_thing != thing {
                belt.marked_for_thing = thing;
            }
        }

        if let Some(mut tree) = tree {
            let mut mark = false;

            for sign in signs.iter() {
                if sign.thing == Some(Thing::Wood) {
                    mark = true;
                }
            }

            if tree.mark_cut_tree != mark {
                tree.mark_cut_tree = mark;
            }
        }
    }
}
