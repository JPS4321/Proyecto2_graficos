use crate::texture::Texture;
use std::rc::Rc;
use crate::color::Color;

#[derive(Debug, Clone)]
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub albedo: [f32; 4],
    pub refractive_index: f32,
    pub texture: Option<Rc<Texture>>,  // Textura opcional
    pub normal_map: Option<Rc<Texture>>,  // Normal map opcional
    pub emission: Option<Color>,
}

impl Material {
    pub fn new(
        diffuse: Color,
        specular: f32,
        albedo: [f32; 4],
        refractive_index: f32,
        texture: Option<Rc<Texture>>,  // Soporte para texturas
        normal_map: Option<Rc<Texture>>,  // Soporte para normal maps
        emission: Option<Color>,  // Color de emisión (para objetos que emiten luz)
    ) -> Self {
        Material {
            diffuse,
            specular,
            albedo,
            refractive_index,
            texture,
            normal_map,
            emission,
        }
    }

    pub fn black() -> Self {
        Material {
            diffuse: Color::black(),  // Usar el nuevo método black() que devuelve f32
            specular: 0.0,
            albedo: [0.0, 0.0, 0.0, 0.0],
            refractive_index: 0.0,
            texture: None,
            normal_map: None,
            emission: None,
        }
    }
}
