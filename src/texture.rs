use image::GenericImageView;

pub struct Texture {
    pub image: image::DynamicImage,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn new(file_path: &str) -> Texture {
        let img = image::open(file_path).expect(&format!("Error al cargar textura {}", file_path));
        let (width, height) = img.dimensions();
        Texture {
            image: img,
            width,
            height,
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if x >= self.width || y >= self.height {
            return 0;
        }
        let pixel = self.image.get_pixel(x, y);
        let r = pixel[0] as u32;
        let g = pixel[1] as u32;
        let b = pixel[2] as u32;
        let a = pixel[3] as u32; // Alpha channel
        
        (a << 24) | (r << 16) | (g << 8) | b
    }

    pub fn get_tinted_pixel(&self, x: u32, y: u32, tint: (f32, f32, f32)) -> u32 {
        let pixel = self.get_pixel(x, y);
        let r = (((pixel >> 16) & 0xFF) as f32 * tint.0).min(255.0) as u32;
        let g = (((pixel >> 8) & 0xFF) as f32 * tint.1).min(255.0) as u32;
        let b = ((pixel & 0xFF) as f32 * tint.2).min(255.0) as u32;

        (r << 16) | (g << 8) | b
    }
}
