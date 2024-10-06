use std::sync::Arc;
use crate::hittable::hittable::Hittable;

pub struct World {
    pub objects: Box<dyn Hittable>,
    pub lights: Arc<dyn Hittable>
}

impl World {
    pub fn new(objects: Box<dyn Hittable>, lights: Arc<dyn Hittable>) -> Self {
        World { objects, lights }
    }
}