
use nalgebra_glm::Vec3;
use crate::color::Color;

#[derive(Clone)]  // Esto automáticamente implementa el trait Clone para la estructura Light
pub struct Light {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
}

impl Light {
    pub fn new(position: Vec3, color: Color, intensity: f32) -> Self {
        Light {
            position,
            color,
            intensity,
        }
    }
}
