use crate::hittable::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::material::{Isotropic, Material};
use crate::ray::Ray;
use crate::vec3::Vec3;
use crate::AABB::AABB;
use std::sync::Arc;

pub struct ConstantMedium {
	boundary: Box<dyn Hittable>,
	density_neg_inv: f64,
	phase_function: Arc<dyn Material>
}

impl ConstantMedium {
	pub fn new(boundary: Box<dyn Hittable>, density: f64, phase_function: Arc<dyn Material>) -> Self {
		let density_neg_inv = -1.0 / density;

		ConstantMedium { boundary, density_neg_inv, phase_function }
	}

	pub fn from_isotropic_color(boundary: Box<dyn Hittable>, density: f64, color: Vec3) -> Self {
		let density_neg_inv = - 1.0 / density;
		let phase_function = Arc::new(Isotropic::from_color(color));

		ConstantMedium { boundary, density_neg_inv, phase_function}
	}
}

impl Hittable for ConstantMedium {
	fn hit(&self, ray: Ray, ray_t: Interval) -> Option<HitRecord> {
		let mut hit_record_1 = self.boundary.hit(ray, Interval::UNIVERSE)?;
		let mut hit_record_2 = self.boundary.hit(
			ray,
			Interval::new(hit_record_1.t + 0.0001, f64::MAX)
		)?;

		if hit_record_1.t < ray_t.min { hit_record_1.t = ray_t.min; }
		if hit_record_2.t > ray_t.max { hit_record_2.t = ray_t.max; }

		if hit_record_1.t >= hit_record_2.t { return None; }

		if hit_record_1.t < 0.0 { hit_record_1.t = 0.0; }

		let ray_len = ray.direction.length();
		let dist_inside_boundary = (hit_record_2.t - hit_record_1.t) * ray_len;
		let hit_dist = self.density_neg_inv * f64::ln(fastrand::f64());

		if hit_dist > dist_inside_boundary { return None; }

		let t = hit_record_1.t + hit_dist / ray_len;

		Some(HitRecord::new(
			ray,
			t,
			ray.at(t),
			Vec3::new(1.0, 0.0, 0.0),
			self.phase_function.clone(),
			0.0,
			0.0
		))
	}

	fn bounding_box(&self) -> AABB {
		self.boundary.bounding_box()
	}
}