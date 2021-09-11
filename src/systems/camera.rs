use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_mod_picking::PickingCameraBundle;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(camera_tracking);
    }
}

fn camera_tracking(
    time: Res<Time>,
    mut camera: Query<&mut Transform, With<MyCamera>>,
    tracking: Query<(&Transform, &CameraTracking), Without<MyCamera>>,
) {
    let dt = time.delta_seconds();

    for (tracking_transform, tracking) in tracking.get_single() {
        for mut camera_transform in camera.get_single_mut() {
            let diff =
                tracking_transform.translation + tracking.offset - camera_transform.translation;
            let step = diff.normalize_or_zero() * diff.length_squared();
            *camera_transform =
                Transform::from_translation(camera_transform.translation + step * dt)
                    .looking_at(tracking_transform.translation, Vec3::Y);
        }
    }
}

struct MyCamera;

pub struct CameraTracking {
    offset: Vec3,
}

impl CameraTracking {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            offset: Vec3::new(x, y, z),
        }
    }
}

#[derive(SystemParam)]
pub struct CameraSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
}

impl<'w, 's> CameraSpawn<'w, 's> {
    pub fn spawn(&mut self, center: Vec3) -> Entity {
        let Self { cmds, .. } = self;
        cmds.spawn().insert(DirectionalLight::new(
            Color::WHITE,
            25000.0,
            Vec3::new(1.0, -1.0, 0.5).normalize(),
        ));

        cmds.spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(center.x, 10.0, center.z - 5.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert_bundle(PickingCameraBundle::default())
        .insert(MyCamera)
        .id()
    }
}
