use bevy::math::Vec3;
use lyon::{geom::point, geom::Point};

pub trait ToPoint {
    fn to_point(&self) -> Point<f32>;
}

impl ToPoint for Vec3 {
    fn to_point(&self) -> Point<f32> {
        point(self.x, self.z)
    }
}

pub trait ToVec3 {
    fn to_vec3(&self) -> Vec3;
}

impl ToVec3 for Point<f32> {
    fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x, 0.0, self.y)
    }
}
