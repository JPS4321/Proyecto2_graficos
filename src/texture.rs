use image::{DynamicImage, GenericImageView};
use std::rc::Rc;

#[derive(Debug)] 
pub struct Texture {
    image_data: DynamicImage,
}

impl Texture {
    pub fn new(image_path: &str) -> Self {
        match image::open(image_path) {
            Ok(image_data) => Texture { image_data },
            Err(e) => {
                panic!("Error al cargar la textura desde {}: {}", image_path, e);
            }
        }
    }

    pub fn width(&self) -> u32 {
        self.image_data.width()
    }

    pub fn height(&self) -> u32 {
        self.image_data.height()
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 3] {
        let pixel = self.image_data.get_pixel(x, y);
        [pixel[0], pixel[1], pixel[2]]  // Retorna solo los valores RGB
    }
}

pub fn load_texture(file_path: &str) -> Rc<Texture> {
    Rc::new(Texture::new(file_path))
}
