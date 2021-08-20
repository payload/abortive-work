use std::f32::consts::TAU;

use bevy::{ecs::system::SystemParam, math::vec3, prelude::*};

pub struct Imp {
    idle_time: f32,
    walk_to: Vec3,
}

impl Imp {
    pub fn new() -> Self {
        Self {
            idle_time: 0.0,
            walk_to: Vec3::ZERO,
        }
    }
}

#[derive(Clone)]
pub struct ImpAssets {
    transform: Transform,
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
}

pub struct ImpPlugin;

impl Plugin for ImpPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets)
            .add_system(update_imp.label("imp"))
            .add_system(update_walk.after("imp"));
    }
}

#[derive(SystemParam)]
pub struct ImpSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, ImpAssets>,
}

impl<'w, 's> ImpSpawn<'w, 's> {
    pub fn spawn(&mut self, imp: Imp, transform: Transform) {
        let pbr_bundle = PbrBundle {
            transform: self.assets.transform.clone(),
            material: self.assets.material.clone(),
            mesh: self.assets.mesh.clone(),
            ..Default::default()
        };

        self.cmds
            .spawn_bundle((imp, transform, GlobalTransform::identity()))
            .with_children(|p| {
                p.spawn_bundle(pbr_bundle);
            });
    }
}

fn load_assets(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.insert_resource(ImpAssets {
        transform: Transform::from_xyz(0.0, 0.3, 0.0),
        material: materials.add(material(Color::SALMON)),
        mesh: meshes.add(shape::Box::new(0.4, 0.6, 0.4).into()),
    });

    fn material(color: Color) -> StandardMaterial {
        StandardMaterial {
            unlit: true,
            base_color: color,
            ..Default::default()
        }
    }
}

fn update_imp(time: Res<Time>, mut imps: Query<(Entity, &mut Imp, &Transform)>) {
    let now = time.time_since_startup().as_secs_f32();

    for (_entity, mut imp, transform) in imps.iter_mut() {
        if imp.idle_time <= now {
            imp.idle_time = now + 1.0;

            let a = TAU * fastrand::f32();
            let random_offset = vec3(a.cos(), 0.0, a.sin());
            imp.walk_to = transform.translation + random_offset;
        }
    }
}

fn update_walk(time: Res<Time>, mut imps: Query<(&Imp, &mut Transform)>) {
    let dt = time.delta_seconds();

    for (imp, mut transform) in imps.iter_mut() {
        let diff = imp.walk_to - transform.translation;
        let len2 = diff.length_squared();
        let vec = if len2 < 1.0 { diff } else { diff / len2 / len2 };
        let speed = 3.0;
        let step = vec * speed * dt;
        transform.translation += step;
    }
}
