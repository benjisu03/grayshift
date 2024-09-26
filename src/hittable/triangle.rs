use std::sync::Arc;
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
    pub fn new(a: Vec3, b: Vec3, c: Vec3, material: Arc<dyn Material>) -> Self {
        let normal = (b - a).cross(c - a);

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
        if det < Self::EPSILON { return None; }

        let t_vec = ray.origin - self.a;
        let mut u = t_vec.dot(p_vec);
        if u < 0.0 || u > det { return None; }

        let q_vec = t_vec.cross(edge_1);
        let mut v = ray.direction.dot(q_vec);
        if v < 0.0 || u + v > det { return None; }

        let mut t = edge_2.dot(q_vec);
        let inv_det = 1.0 / det;

        t *= inv_det;
        u *= inv_det;
        v *= inv_det;

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
