use std::sync::Arc;
use crate::AABB::AABB;
use crate::hittable::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::util::interval::Interval;
use crate::util::vec3::Vec3;

pub struct Plane {
    pub normal: Vec3,
    pub d: f64,
}

impl Plane {
    pub fn new(normal: Vec3, origin: Vec3) -> Self {
        let d = normal.dot(origin);
        Plane { normal, d }
    }

    pub fn hit(&self, ray: Ray, ray_t: Interval) -> Option<(f64, Vec3)> {
        let denominator = self.normal.dot(ray.direction);

        // check if ray is parallel to the plane
        if denominator.abs() < 1e-8 { return None; }

        let t = (self.d - self.normal.dot(ray.origin)) / denominator;
        if !ray_t.contains(t) { return None; }

        let intersection = ray.at(t);

        Some((t, intersection))
    }
}