use bevy::{
    ecs::system::{EntityCommands, SystemParam},
    prelude::*,
};

use super::NotGround;

#[derive(Default)]
pub struct Conveyor {}

impl Conveyor {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct ConveyorPlugin;

impl Plugin for ConveyorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets)
            .add_system_to_stage(CoreStage::Update, update_lines);
    }
}

#[derive(SystemParam)]
pub struct ConveyorSpawn<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, ConveyorAssets>,
}

impl<'w, 's> ConveyorSpawn<'w, 's> {
    pub fn spawn_line<'a>(&'a mut self, from: Vec3, to: Vec3) {
        for (pos, angle) in Self::spawn_positions_at_line(from, to) {
            self.spawn(
                Conveyor::new(),
                Transform {
                    rotation: Quat::from_rotation_y(angle),
                    translation: pos,
                    ..Default::default()
                },
            );
        }
    }

    pub fn ghostline_from_point_to_entity(&mut self, from: Vec3, to: Entity) -> Entity {
        self.cmds
            .spawn_bundle((
                LineFrom { to },
                Transform::from_translation(from),
                GlobalTransform::identity(),
            ))
            .id()
    }

    pub fn spawn_ghostline(&mut self, from: Vec3, to: Vec3, parent: Entity) {
        for (pos, angle) in Self::spawn_positions_at_line(from, to) {
            self.spawn_ghost(pos, angle, parent);
        }
    }

    fn spawn_positions_at_line(from: Vec3, to: Vec3) -> impl Iterator<Item = (Vec3, f32)> {
        let way = to - from;
        let dir = way.normalize();
        let angle = dir.x.atan2(dir.z);
        let steps = way.length().floor() as i32;
        (0..=steps).map(move |step| {
            let step = step as f32;
            let pos = from + dir * step;
            (pos, angle)
        })
    }

    fn spawn_ghost(&mut self, pos: Vec3, angle: f32, parent: Entity) {
        let model = self.model(self.assets.ghost_material.clone());
        let ghost = self
            .cmds
            .spawn_bundle((
                GlobalTransform::identity(),
                Transform {
                    rotation: Quat::from_rotation_y(angle),
                    translation: pos,
                    ..Default::default()
                },
            ))
            .push_children(&[model])
            .id();
        self.cmds.entity(parent).push_children(&[ghost]);
    }

    fn model(&mut self, material: Handle<StandardMaterial>) -> Entity {
        self.cmds
            .spawn_bundle(PbrBundle {
                material,
                mesh: self.assets.mesh.clone(),
                transform: self.assets.transform.clone(),
                ..Default::default()
            })
            .insert(NotGround)
            .id()
    }

    fn spawn<'a>(
        &'a mut self,
        conveyor: Conveyor,
        transform: Transform,
    ) -> EntityCommands<'w, 's, 'a> {
        let model = self.model(self.assets.material.clone());
        let mut entity_cmds =
            self.cmds
                .spawn_bundle((conveyor, transform, GlobalTransform::identity()));
        entity_cmds.push_children(&[model]);
        entity_cmds
    }
}

#[derive(Clone)]
pub struct ConveyorAssets {
    pub transform: Transform,
    pub material: Handle<StandardMaterial>,
    pub ghost_material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
}

fn load_assets(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.insert_resource(ConveyorAssets {
        transform: Transform::from_xyz(0.0, 0.25, 0.0),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.1, 0.1, 0.1),
            metallic: 0.0,
            roughness: 0.0,
            reflectance: 0.0,
            ..Default::default()
        }),
        ghost_material: materials.add(StandardMaterial {
            base_color: Color::rgba(0.1, 0.1, 0.1, 0.4),
            metallic: 0.0,
            roughness: 0.0,
            reflectance: 0.0,
            ..Default::default()
        }),
        mesh: meshes.add(shape::Box::new(0.7, 0.1, 0.95).into()),
    });
}

struct LineFrom {
    to: Entity,
}

struct LineParent;

fn update_lines(
    mut frame: Local<usize>,
    mut cmds: Commands,
    mut conveyor: ConveyorSpawn,
    ghostlines: Query<(&Transform, &LineFrom)>,
    transforms: Query<&Transform>,
    parents: Query<Entity, With<LineParent>>,
) {
    if *frame < 2 {
        *frame += 1;
        return;
    }

    *frame = 0;

    for parent in parents.iter() {
        cmds.entity(parent).despawn_recursive();
    }

    for (from_transform, ghostline) in ghostlines.iter() {
        if let Ok(to_transform) = transforms.get(ghostline.to) {
            let parent = cmds
                .spawn_bundle((
                    LineParent,
                    Transform::identity(),
                    GlobalTransform::identity(),
                ))
                .id();
            conveyor.spawn_ghostline(from_transform.translation, to_transform.translation, parent);
        }
    }
}
