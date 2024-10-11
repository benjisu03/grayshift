use crate::hittable::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::util::interval::Interval;
use crate::AABB::AABB;
use nalgebra::Vector3;
use std::f32;
use std::sync::Arc;

pub struct Sphere {
	center_start: Vector3<f32>,
	center_path: Vector3<f32>,
	is_moving: bool,
	radius: f32,
	material: Arc<dyn Material>,
	bbox: AABB
}

impl Sphere {
	pub fn new_stationary(center: Vector3<f32>, radius: f32, material: Arc<dyn Material>) -> Self {
		let r_vec = Vector3::new(radius, radius, radius);
		let bbox = AABB::from_corners(center - r_vec, center + r_vec);

		Sphere {
			center_start: center,
			center_path: Vector3::new(0.0, 0.0, 0.0),
			is_moving: false,
			radius,
			material,
			bbox
		}
	}

	pub fn new_moving(center1: Vector3<f32>, center2: Vector3<f32>, radius: f32, material: Arc<dyn  Material>) -> Self {
		let r_vec = Vector3::new(radius, radius, radius);
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

	fn current_center(&self, time: f32) -> Vector3<f32> {
		self.center_start + time * self.center_path
	}

	fn sphere_uv(p: Vector3<f32>) -> (f32, f32) {
		let theta = (-p.y).acos();
		let phi = (-p.z).atan2(p.x) + f32::consts::PI;

		(phi / (2.0 * f32::consts::PI), theta / f32::consts::PI)
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
		let a = ray.direction.magnitude_squared();
		let h = ray.direction.dot(&oc);
		let c = oc.magnitude_squared() - self.radius * self.radius;
		let discriminant = h * h - a * c;

		if discriminant < 0.0 {
			return None
		}

		let sqrt_d = f32::sqrt(discriminant);

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