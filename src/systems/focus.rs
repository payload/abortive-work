use std::{cmp::Ordering, f32::consts::FRAC_PI_4};

use bevy::{prelude::*, render::camera::Camera};

#[derive(Default)]
pub struct Focus {
    pub entity: Option<Entity>,
    pub before: Option<Entity>,
    pub screen_pos: Option<Vec2>,
}

#[derive(Default)]
pub struct FocusObject {
    pub focussed: bool,
}

impl FocusObject {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct FocusPlugin;

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::Update, update_focus);
    }
}

fn update_focus(
    mut foci: Query<(&mut Focus, &Transform)>,
    objects: Query<(Entity, &Transform), With<FocusObject>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    windows: Res<Windows>,
) {
    let (camera, camera_transform) = camera.single();

    for (mut focus, focus_transform) in foci.iter_mut() {
        let mut candidates = Vec::new();

        for (object, object_transform) in objects.iter() {
            let diff = object_transform.translation - focus_transform.translation;
            let distance = diff.length_squared();

            if distance < 4.0 {
                let look = focus_transform.rotation * Vec3::Z;
                let angle = look.angle_between(diff);

                if angle < FRAC_PI_4 {
                    candidates.push((object, distance));
                }
            }
        }

        candidates.sort_unstable_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Less));

        let candidate = candidates.first().map(|(e, _)| *e);
        if focus.entity != candidate {
            focus.before = focus.entity;
            focus.entity = candidate;
        }

        if let Some(entity) = focus.entity {
            let world_position = objects.get(entity).unwrap().1.translation
                // TODO: add offset to left because I don't know how to center the UI on that point
                // TODO: add offset to top because things are often higher and the pos is at the ground
                + Vec3::new(0.5, 2.5, 0.0);
            focus.screen_pos = camera.world_to_screen(&windows, camera_transform, world_position);
        } else {
            focus.screen_pos = None;
        }
    }
}
