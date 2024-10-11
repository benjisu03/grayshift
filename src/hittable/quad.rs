use crate::hittable::hittable::{HitRecord, Hittable, HittableList};
use crate::hittable::plane::Plane;
use crate::material::Material;
use crate::ray::Ray;
use crate::util::interval::Interval;
use crate::AABB::AABB;
use nalgebra::Vector3;
use std::sync::Arc;

pub struct Quad {
	plane: Plane,
	area: f32,

	q: Vector3<f32>,
	u: Vector3<f32>,
	v: Vector3<f32>,
	w: Vector3<f32>,

	material: Arc<dyn Material>,
	bbox: AABB,
}

impl Quad {
	pub fn new(q: Vector3<f32>, u: Vector3<f32>, v: Vector3<f32>, material: Arc<dyn Material>) -> Self {
		let bbox_diag1 = AABB::from_corners(q, q + u + v);
		let bbox_diag2 = AABB::from_corners(q + u, q + v);
		let bbox = AABB::from_AABB_pair(bbox_diag1, bbox_diag2);

		let n = u.cross(&v);
		let normal = n.normalize();

		let w = n / n.dot(&n);

		let plane = Plane::new(normal, q);
		let area = n.magnitude();

		Quad { plane, area, q, u, v, w, material, bbox }
	}

	pub fn is_in_mandelbrot(alpha: f32, beta: f32, max_iterations: usize) -> bool {
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

	pub fn cube(point_a: Vector3<f32>, point_b: Vector3<f32>, material: Arc<dyn Material>) -> HittableList {
		let mut sides = HittableList::new();

		let min = Vector3::new(
			point_a.x.min(point_b.x),
			point_a.y.min(point_b.y),
			point_a.z.min(point_b.z)
		);
		let max = Vector3::new(
			point_a.x.max(point_b.x),
			point_a.y.max(point_b.y),
			point_a.z.max(point_b.z)
		);

		let dx = Vector3::new(max.x - min.x, 0.0, 0.0);
		let dy = Vector3::new(0.0, max.y - min.y, 0.0);
		let dz = Vector3::new(0.0, 0.0, max.z - min.z);

		sides.add(Box::new(Quad::new(Vector3::new(min.x, min.y, max.z),  dx,  dy, material.clone())));
		sides.add(Box::new(Quad::new(Vector3::new(max.x, min.y, max.z), -dz,  dy, material.clone())));
		sides.add(Box::new(Quad::new(Vector3::new(max.x, min.y, min.z), -dx,  dy, material.clone())));
		sides.add(Box::new(Quad::new(Vector3::new(min.x, min.y, min.z),  dz,  dy, material.clone())));
		sides.add(Box::new(Quad::new(Vector3::new(min.x, max.y, max.z),  dx, -dz, material.clone())));
		sides.add(Box::new(Quad::new(Vector3::new(min.x, min.y, min.z),  dx,  dz, material)));

		sides
	}
}

impl Hittable for Quad {
	fn hit(&self, ray: Ray, ray_t: Interval) -> Option<HitRecord> {
		let (t, intersection) = self.plane.hit(ray, ray_t)?;

		let planar_hit = intersection - self.q;
		let alpha = self.w.dot(&planar_hit.cross(&self.v));
		let beta = self.w.dot(&self.u.cross(&planar_hit));

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

	fn pdf_value(&self, origin: Vector3<f32>, direction: Vector3<f32>) -> f32 {
		if let Some(hit_record) = self.hit(Ray::new(origin, direction, 0.0), Interval::new(0.001, f32::MAX)) {
			let dist_sq = hit_record.t * hit_record.t * direction.magnitude_squared();
			let cosine = (direction.dot(&hit_record.normal) / direction.magnitude()).abs();
			return dist_sq / (cosine * self.area);
		}

		0.0
	}

	fn random(&self, origin: Vector3<f32>) -> Vector3<f32> {
		let p = self.q + (fastrand::f32() * self.u) + (fastrand::f32() * self.v);
		p - origin
	}
}