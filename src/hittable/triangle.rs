use std::sync::Arc;
use crate::AABB::AABB;
use crate::hittable::hittable::{HitRecord, Hittable};
use crate::hittable::plane::Plane;
use crate::material::Material;
use crate::ray::Ray;
use crate::util::interval::Interval;
use crate::util::vec3::Vec3;

pub struct Triangle {
    plane: Plane,
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
        let plane = Plane::new(normal.unit(), a);
        let normal_sq = normal.length_squared();

        let bbox_diag1 = AABB::from_corners(a, b);
        let bbox_diag2 = AABB::from_corners(a, c);
        let bbox = AABB::from_AABB_pair(bbox_diag1, bbox_diag2);

        Triangle {
            plane,
            normal,
            a,
            b,
            c,
            material,
            bbox
        }
    }
}

impl Hittable for Triangle {
    fn hit(&self, ray: Ray, ray_t: Interval) -> Option<HitRecord> {
        let (t, p) = self.plane.hit(ray, ray_t)?;

        let normal_sq = self.normal.length_squared();

        let na = (self.c - self.b).cross(p - self.b);
        let nb = (self.a - self.c).cross(p - self.c);
        let nc = (self.b - self.a).cross(p - self.a);

        let alpha = na.dot(self.normal) / normal_sq;
        let beta =  nb.dot(self.normal) / normal_sq;
        let gamma = nc.dot(self.normal) / normal_sq;

        if alpha < 0.0 || beta < 0.0 || gamma < 0.0 {
            // point lies on plane but outside of triangle
            return None;
        }

        Some(HitRecord::new(
            ray,
            t,
            p,
            self.plane.normal,
            self.material.clone(),
            0.0, // TODO: not sure what to put for these
            0.0  //
        ))
    }

    fn bounding_box(&self) -> AABB {
        self.bbox
    }
}
