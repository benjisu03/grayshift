use std::error::Error;
use std::fs::File;
use std::path::Path;
use crate::camera::Camera;
use crate::color::write_color;
use crate::output::RenderTarget;
use crate::util::vec3::Vec3;

pub struct Engine {
    camera: Camera,
    render_target: Box<dyn RenderTarget>
}

impl Engine {
    pub fn new(camera: Camera, render_target: Box<dyn RenderTarget>) -> Engine {

        Engine { camera, render_target }
    }
}

