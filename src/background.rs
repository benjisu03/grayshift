use std::error::Error;
use std::f64::consts::PI;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use crate::util::util::rotate_vector;
use crate::util::vec3::Vec3;

pub trait Background: Send + Sync {
    fn sample(&self, direction: Vec3) -> Vec3;
}

pub struct SolidColorBackground {
    color: Vec3
}

impl SolidColorBackground {
    pub fn new(color: Vec3) -> SolidColorBackground {
        SolidColorBackground { color }
    }
}

impl Background for SolidColorBackground {
    fn sample(&self, direction: Vec3) -> Vec3 {
        self.color
    }
}

pub struct HDRIBackground {
    pub image: radiant::Image,
    pub rotation: Vec3
}

impl HDRIBackground {
    pub fn new<P: AsRef<Path>>(image_path: P, rotation: Vec3) -> Result<Self, Box<dyn Error>> {
        let file = File::open(&image_path)?;
        let buffer = BufReader::new(file);
        let image = radiant::load(buffer)?;

        Ok(HDRIBackground { image, rotation })
    }
}

impl Background for HDRIBackground {
    fn sample(&self, direction: Vec3) -> Vec3 {
        let rotated = rotate_vector(direction, self.rotation).unit();
        let theta = rotated.y.atan2(rotated.x);
        let phi = rotated.z.asin();

        let u = 0.5 + theta / (2.0 * PI);
        let v = 0.5 - phi / PI;

        let x = ((u * (self.image.width as f64)) as usize) % self.image.width;
        let y = ((v * (self.image.height as f64)) as usize) % self.image.height;

        let color = self.image.pixel(x, y);
        Vec3::new(color.r as f64, color.g as f64, color.b as f64)
    }
}