use bevy::{ecs::system::SystemParam, prelude::*, transform::TransformSystem};
use bevy_mod_picking::PickingCameraBundle;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PostUpdate, camera_tracking)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                look_at_camera.after(TransformSystem::TransformPropagate),
            );
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
            let target = tracking_transform.translation + tracking.offset;
            camera_transform.translation = camera_transform
                .translation
                .lerp(target, (dt * 3.0).min(1.0));
            let target_rot = camera_transform
                .looking_at(tracking_transform.translation, Vec3::Y)
                .rotation;
            camera_transform.rotation = camera_transform.rotation.lerp(target_rot, dt.min(1.0));
        }
    }
}

struct MyCamera;

pub struct CameraTracking {
    pub offset: Vec3,
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

pub struct LookAtCamera;

fn look_at_camera(
    mut query: Query<&mut GlobalTransform, With<LookAtCamera>>,
    camera: Query<&GlobalTransform, (With<MyCamera>, Without<LookAtCamera>)>,
) {
    let camera_global_t = camera.single();

    for mut global_t in query.iter_mut() {
        global_t.rotation = camera_global_t.rotation.inverse();
    }
}
