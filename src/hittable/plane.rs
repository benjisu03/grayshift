use crate::ray::Ray;
use crate::util::interval::Interval;
use nalgebra::Vector3;

pub struct Plane {
    pub normal: Vector3<f32>,
    pub d: f32,
}

impl Plane {
    pub fn new(normal: Vector3<f32>, origin: Vector3<f32>) -> Self {
        let d = normal.dot(&origin);
        Plane { normal, d }
    }

    pub fn hit(&self, ray: Ray, ray_t: Interval) -> Option<(f32, Vector3<f32>)> {
        let denominator = self.normal.dot(&ray.direction);

        // check if ray is parallel to the plane
        if denominator.abs() < 1e-8 { return None; }

        let t = (self.d - self.normal.dot(&ray.origin)) / denominator;
        if !ray_t.contains(t) { return None; }

        let intersection = ray.at(t);

        Some((t, intersection))
    }
}