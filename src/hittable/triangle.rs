use std::sync::Arc;
use log::error;
use crate::AABB::AABB;
use crate::hittable::hittable::{HitRecord, Hittable};
use crate::hittable::plane::Plane;
use crate::material::Material;
use crate::ray::Ray;
use crate::util::interval::Interval;
use crate::util::vec3::Vec3;

pub struct Triangle {
    normal: Vec3,
    a: Vec3,
    b: Vec3,
    c: Vec3,
    material: Arc<dyn Material>,
    bbox: AABB,
}

impl Triangle {
    pub fn new(a: Vec3, b: Vec3, c: Vec3, normal: Vec3, material: Arc<dyn Material>) -> Self {

        let bbox_diag1 = AABB::from_corners(a, b);
        let bbox_diag2 = AABB::from_corners(a, c);
        let bbox = AABB::from_AABB_pair(bbox_diag1, bbox_diag2);

        Triangle { normal, a, b, c, material, bbox }
    }

    const EPSILON: f64 = 1e-8;
}

impl Hittable for Triangle {
    fn hit(&self, ray: Ray, ray_t: Interval) -> Option<HitRecord> {
        let edge_1 = self.c - self.a;
        let edge_2 = self.b - self.a;

        let p_vec = ray.direction.cross(edge_2);
        let det = edge_1.dot(p_vec);
        if -Self::EPSILON < det && det < Self::EPSILON  { return None; }

        let inv_det = 1.0 / det;

        let t_vec = ray.origin - self.a;
        let u = t_vec.dot(p_vec) * inv_det;
        if u < 0.0 || u > 1.0 { return None; }

        let q_vec = t_vec.cross(edge_1);
        let v = ray.direction.dot(q_vec) * inv_det;
        if v < 0.0 || u + v > 1.0 { return None; }

        let t = edge_2.dot(q_vec) * inv_det;
        if t < ray_t.min || t > ray_t.max { return None; }

        let pos = ray.at(t);

        Some(HitRecord::new(
            ray,
            t,
            pos,
            self.normal,
            self.material.clone(),
            u,
            v
        ))
    }

    fn bounding_box(&self) -> AABB {
        self.bbox
    }
}
