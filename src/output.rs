use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use nalgebra::Vector3;
use crate::color::write_color;

pub trait RenderTarget: Send + Sync {
    fn init(&mut self) -> Result<(), Box<dyn Error>>;
    fn write_color(&mut self, color: Vector3<f32>) -> Result<(), Box<dyn Error>>;
    fn size(&self) -> (u32, u32);
}


pub struct PPMImage {
    file: File,
    width: u32,
    height: u32
}

impl PPMImage {
    pub fn new<P: AsRef<Path>>(p: P, width: u32, height: u32) -> std::io::Result<Self> {
        let file = File::create(p)?;
        Ok(PPMImage { file, width, height })
    }
}

impl RenderTarget for PPMImage {
    fn init(&mut self) -> Result<(), Box<dyn Error>> {
        writeln!(self.file, "P3")?;
        writeln!(self.file, "{} {}", self.width, self.height)?;
        writeln!(self.file, "255")?;
        Ok(())
    }

    fn write_color(&mut self, color: Vector3<f32>) -> Result<(), Box<dyn Error>> {
        write_color(&mut self.file, color)?;
        Ok(())
    }

    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}