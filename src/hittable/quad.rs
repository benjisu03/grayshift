use std::rc::Rc;
use std::sync::Arc;
use log::warn;
use crate::AABB::AABB;
use crate::hittable::hittable::{HitRecord, Hittable, HittableList};
use crate::hittable::plane::Plane;
use crate::material::Material;
use crate::ray::Ray;
use crate::util::interval::Interval;
use crate::util::vec3::Vec3;

pub struct Quad {
	plane: Plane,

	q: Vec3,
	u: Vec3,
	v: Vec3,
	w: Vec3,

	material: Arc<dyn Material>,
	bbox: AABB,
}

impl Quad {
	pub fn new(q: Vec3, u: Vec3, v: Vec3, material: Arc<dyn Material>) -> Self {
		let bbox_diag1 = AABB::from_corners(q, q + u + v);
		let bbox_diag2 = AABB::from_corners(q + u, q + v);
		let bbox = AABB::from_AABB_pair(bbox_diag1, bbox_diag2);

		let n = u.cross(v);
		let normal = n.unit();

		let w = n / n.dot(n);

		let plane = Plane::new(normal, q);

		Quad { plane, q, u, v, w, material, bbox }
	}

	pub fn is_in_mandelbrot(alpha: f64, beta: f64, max_iterations: usize) -> bool {
		let mut z = (0.0, 0.0);
		for _ in 0..max_iterations {
			z = (
				z.0 * z.0 - z.1 * z.1 + alpha,
				2.0 * z.0 * z.1 + beta
			);
			if z.0 * z.0 + z.1 * z.1 > 4.0 {
				return false;
			}
		}
		true
	}

	pub fn cube(point_a: Vec3, point_b: Vec3, material: Arc<dyn Material>) -> HittableList {
		let mut sides = HittableList::new();

		let min = Vec3::new(
			point_a.x.min(point_b.x),
			point_a.y.min(point_b.y),
			point_a.z.min(point_b.z)
		);
		let max = Vec3::new(
			point_a.x.max(point_b.x),
			point_a.y.max(point_b.y),
			point_a.z.max(point_b.z)
		);

		let dx = Vec3::new(max.x - min.x, 0.0, 0.0);
		let dy = Vec3::new(0.0, max.y - min.y, 0.0);
		let dz = Vec3::new(0.0, 0.0, max.z - min.z);

		sides.add(Box::new(Quad::new(Vec3::new(min.x, min.y, max.z),  dx,  dy, material.clone())));
		sides.add(Box::new(Quad::new(Vec3::new(max.x, min.y, max.z), -dz,  dy, material.clone())));
		sides.add(Box::new(Quad::new(Vec3::new(max.x, min.y, min.z), -dx,  dy, material.clone())));
		sides.add(Box::new(Quad::new(Vec3::new(min.x, min.y, min.z),  dz,  dy, material.clone())));
		sides.add(Box::new(Quad::new(Vec3::new(min.x, max.y, max.z),  dx, -dz, material.clone())));
		sides.add(Box::new(Quad::new(Vec3::new(min.x, min.y, min.z),  dx,  dz, material)));

		sides
	}
}

impl Hittable for Quad {
	fn hit(&self, ray: Ray, ray_t: Interval) -> Option<HitRecord> {
		let (t, intersection) = self.plane.hit(ray, ray_t)?;

		let planar_hit = intersection - self.q;
		let alpha = self.w.dot(planar_hit.cross(self.v));
		let beta = self.w.dot(self.u.cross(planar_hit));

		if !Interval::UNIT.contains(alpha) || !Interval::UNIT.contains(beta) {
			return None;
		}

		// cool mandelbrot shape
		// let u = -2.0 + alpha * 3.0;
		// let v = -1.5 + beta * 3.0;
		// if !Quad::is_in_mandelbrot(u, v, 100) { return None; }

		Some(HitRecord::new(
			ray,
			t,
			intersection,
			self.plane.normal,
			self.material.clone(),
			alpha,
			beta
		))
	}

	fn bounding_box(&self) -> AABB {
		self.bbox
	}
}