use std::{cmp::Ordering, f32::consts::FRAC_PI_4};

use bevy::prelude::*;

#[derive(Default)]
pub struct Focus {
    pub entity: Option<Entity>,
    pub before: Option<Entity>,
}

pub struct FocusObject;

pub struct FocusPlugin;

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, update_focus);
    }
}

fn update_focus(
    mut foci: Query<(&mut Focus, &Transform)>,
    objects: Query<(Entity, &Transform), With<FocusObject>>,
) {
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
    }
}
