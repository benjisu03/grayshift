use std::error::Error;
use std::f32::consts::PI;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use nalgebra::Vector3;
use crate::util::util::rotate_vector;

pub trait Background: Send + Sync {
    fn sample(&self, direction: Vector3<f32>) -> Vector3<f32>;
}

pub struct SolidColorBackground {
    color: Vector3<f32>
}

impl SolidColorBackground {
    pub fn new(color: Vector3<f32>) -> SolidColorBackground {
        SolidColorBackground { color }
    }
}

impl Background for SolidColorBackground {
    fn sample(&self, direction: Vector3<f32>) -> Vector3<f32> {
        self.color
    }
}

pub struct HDRIBackground {
    pub image: radiant::Image,
    pub rotation: Vector3<f32>
}

impl HDRIBackground {
    pub fn new<P: AsRef<Path>>(image_path: P, rotation: Vector3<f32>) -> Result<Self, Box<dyn Error>> {
        let file = File::open(&image_path)?;
        let buffer = BufReader::new(file);
        let image = radiant::load(buffer)?;

        Ok(HDRIBackground { image, rotation })
    }
}

impl Background for HDRIBackground {
    fn sample(&self, direction: Vector3<f32>) -> Vector3<f32> {
        let rotated = rotate_vector(direction, self.rotation).normalize();
        let theta = rotated.y.atan2(rotated.x);
        let phi = rotated.z.asin();

        let u = 0.5 + theta / (2.0 * PI);
        let v = 0.5 - phi / PI;

        let x = ((u * (self.image.width as f32)) as usize) % self.image.width;
        let y = ((v * (self.image.height as f32)) as usize) % self.image.height;

        let color = self.image.pixel(x, y);
        Vector3::new(color.r, color.g, color.b)
    }
}