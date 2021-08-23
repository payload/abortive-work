use super::BoulderMaterial;

#[derive(Clone, Copy)]
pub struct Rock {
    pub amount: f32,
    pub material: BoulderMaterial,
}
