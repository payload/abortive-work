use std::f32::consts::TAU;

use bevy::{math::vec3, prelude::*};

pub struct AnimationsPlugin;

impl Plugin for AnimationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_funny_animation);
    }
}

pub struct FunnyAnimation {
    pub offset: f32,
}

fn update_funny_animation(time: Res<Time>, mut query: Query<(&FunnyAnimation, &mut Transform)>) {
    let time_fract = time.seconds_since_startup().fract() as f32;

    for (anim, mut transform) in query.iter_mut() {
        let a = anim.offset + time_fract;
        let x = (a * TAU).cos();
        // transform.scale = vec3(1.0 + 0.2 * x, 1.0, 1.0 - 0.2 * x);
        transform.scale = vec3(1.0, 1.0 + 0.2 * x, 1.0);
    }
}
