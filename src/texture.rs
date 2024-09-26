use image::{DynamicImage, GenericImageView};
use std::rc::Rc;

#[derive(Debug)]  // Aquí añadimos el derive para Debug
pub struct Texture {
    image_data: DynamicImage,
}

impl Texture {
    pub fn new(image_path: &str) -> Self {
        let image_data = image::open(image_path).expect("Failed to load texture");
        Texture { image_data }
    }

    pub fn width(&self) -> u32 {
        self.image_data.width()
    }

    pub fn height(&self) -> u32 {
        self.image_data.height()
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 3] {
        let pixel = self.image_data.get_pixel(x, y);
        [pixel[0], pixel[1], pixel[2]]
    }
}

// Mueve esta función fuera del bloque `impl`
pub fn load_texture(file_path: &str) -> Rc<Texture> {
    Rc::new(Texture::new(file_path))
}
