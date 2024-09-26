use image::{DynamicImage, GenericImageView};
use std::rc::Rc;

#[derive(Debug)]  // Añadimos `Debug` para el debugging
pub struct Texture {
    image_data: DynamicImage,
}

impl Texture {
    // La función `new` ahora maneja el error de forma más robusta
    pub fn new(image_path: &str) -> Self {
        match image::open(image_path) {
            Ok(image_data) => Texture { image_data },
            Err(e) => {
                panic!("Error al cargar la textura desde {}: {}", image_path, e);
            }
        }
    }

    // Devuelve el ancho de la imagen
    pub fn width(&self) -> u32 {
        self.image_data.width()
    }

    // Devuelve la altura de la imagen
    pub fn height(&self) -> u32 {
        self.image_data.height()
    }

    // Obtiene un píxel de la textura, devolviendo un arreglo con los valores RGB
    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 3] {
        let pixel = self.image_data.get_pixel(x, y);
        [pixel[0], pixel[1], pixel[2]]  // Retorna solo los valores RGB
    }
}

// Función para cargar la textura y devolver un Rc<Texture>
pub fn load_texture(file_path: &str) -> Rc<Texture> {
    Rc::new(Texture::new(file_path))
}
