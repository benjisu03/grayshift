use std::f64::consts::PI;
use std::rc::Rc;
use std::sync::Arc;
use crate::AABB::AABB;
use crate::hittable::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::util::interval::Interval;
use crate::util::vec3::Vec3;

pub struct Sphere {
	center_start: Vec3,
	center_path: Vec3,
	is_moving: bool,
	radius: f64,
	material: Rc<dyn Material>,
	bbox: AABB
}

impl Sphere {
	pub fn new_stationary(center: Vec3, radius: f64, material: Rc<dyn Material>) -> Self {
		let r_vec = Vec3::new(radius, radius, radius);
		let bbox = AABB::from_corners(center - r_vec, center + r_vec);

		Sphere {
			center_start: center,
			center_path: Vec3::ZERO,
			is_moving: false,
			radius,
			material,
			bbox
		}
	}

	pub fn new_moving(center1: Vec3, center2: Vec3, radius: f64, material: Rc<dyn  Material>) -> Self {
		let r_vec = Vec3::new(radius, radius, radius);
		let bbox_start = AABB::from_corners(center1 - r_vec, center1 + r_vec);
		let bbox_end = AABB::from_corners(center2 - r_vec, center2 + r_vec);
		let bbox_full = AABB::from_AABB_pair(bbox_start, bbox_end);

		Sphere {
			center_start: center1,
			center_path: center2 - center1,
			is_moving: true,
			radius,
			material,
			bbox: bbox_full
		}
	}

	fn current_center(&self, time: f64) -> Vec3 {
		self.center_start + time * self.center_path
	}

	fn sphere_uv(p: Vec3) -> (f64, f64) {
		let theta = (-p.y).acos();
		let phi = (-p.z).atan2(p.x) + PI;

		(phi / (2.0 * PI), theta / PI)
	}
}

impl Hittable for Sphere {
	fn hit(&self, ray: Ray, ray_t: Interval) -> Option<HitRecord> {
		let center = if self.is_moving {
			self.current_center(ray.time)
		} else {
			self.center_start
		};
		let oc = center - ray.origin;
		let a = ray.direction.length_squared();
		let h = ray.direction.dot(oc);
		let c = oc.length_squared() - self.radius * self.radius;
		let discriminant = h * h - a * c;

		if discriminant < 0.0 {
			return None
		}

		let sqrt_d = f64::sqrt(discriminant);

		// Find root in ray's range
		let mut t = (h - sqrt_d) / a;
		if !ray_t.surrounds(t) {
			t = (h + sqrt_d) / a;
			if !ray_t.surrounds(t) {
				return None
			}
		}

		let hit_pos = ray.at(t);
		let outward_normal = (hit_pos - center) / self.radius;


		let (u, v) = Self::sphere_uv(outward_normal);

		Some(HitRecord::new(
			ray,
			t,
			hit_pos,
			outward_normal,
			self.material.clone(),
			u,
			v
		))
	}

	fn bounding_box(&self) -> AABB {
		self.bbox
	}
}