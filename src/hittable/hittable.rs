use crate::material::Material;
use crate::ray::Ray;
use std::rc::Rc;
use std::sync::Arc;
use crate::AABB::AABB;
use crate::util::interval::Interval;
use crate::util::util::deg_to_rad;
use crate::util::vec3::Vec3;

pub trait Hittable: Send + Sync {
	fn hit(&self, ray: Ray, ray_t: Interval) -> Option<HitRecord>;
	fn bounding_box(&self) -> AABB;
	fn pdf_value(&self, origin: Vec3, direction: Vec3) -> f64 { 0.0 }
	fn random(&self, origin: Vec3) -> Vec3 { Vec3::new(1.0, 0.0, 0.0) }
}

pub struct HitRecord {
	pub t: f64,
	pub position: Vec3,
	pub normal: Vec3,
	pub is_front_face: bool,
	pub material: Arc<dyn Material>,
	pub u: f64,
	pub v: f64
}

impl HitRecord {
	pub fn new(
		ray: Ray,
		t: f64,
		position: Vec3,
		normal: Vec3,
		material: Arc<dyn Material>,
		u: f64,
		v: f64
	) -> Self {
		let is_front_face = ray.direction.dot(normal) < 0.0;
		let flipped_normal = if is_front_face { normal } else { -normal };

		HitRecord {
			t, position, is_front_face, material, u, v,
			normal: flipped_normal
		}
	}
}

pub struct HittableList {
	pub objects: Vec<Box<dyn Hittable>>,
	pub bbox: AABB
}

impl HittableList {
	pub fn new() -> Self {
		HittableList {
			objects: Vec::new(),
			bbox: AABB::EMPTY
		}
	}

	pub fn clear(&mut self) {
		self.objects.clear();
		self.bbox = AABB::EMPTY;
	}

	pub fn add(&mut self, object: Box<dyn Hittable>) {
		self.bbox = AABB::from_AABB_pair(self.bbox, object.bounding_box());
		self.objects.push(object);
	}

	pub fn take_objects(self) -> Vec<Box<dyn Hittable>> {
		self.objects
	}

}

impl Hittable for HittableList {
	fn hit(&self, ray: Ray, ray_t: Interval) -> Option<HitRecord> {
		let mut closest_hit = ray_t.max;
		let mut hit_record =  None;

		for object in self.objects.iter() {
			if let Some(rec) = object.hit(
				ray,
				Interval::new(ray_t.min, closest_hit)
			) {
				closest_hit = rec.t;
				hit_record = Some(rec);
			}
		}

		hit_record
	}

	fn bounding_box(&self) -> AABB {
		self.bbox
	}
}

pub struct Translate {
	object: Box<dyn Hittable>,
	offset: Vec3,
	bbox: AABB
}

impl Translate {
	pub fn new(object: Box<dyn Hittable>, offset : Vec3) -> Self {
		let bbox = object.bounding_box() + offset;
		Translate { object, offset, bbox }
	}
}

impl Hittable for Translate {
	fn hit(&self, ray: Ray, ray_t: Interval) -> Option<HitRecord> {
		let offset_ray = Ray::new(
			ray.origin - self.offset,
			ray.direction,
			ray.time
		);

		if let Some(mut hit_record) = self.object.hit(offset_ray, ray_t) {
			hit_record.position += self.offset;
			return Some(hit_record);
		}

		None
	}

	fn bounding_box(&self) -> AABB {
		self.bbox
	}
}

pub struct RotateY {
	object: Box<dyn Hittable>,
	sin_theta: f64,
	cos_theta: f64,
	bbox: AABB
}

impl RotateY {
	pub fn new(object: Box<dyn Hittable>, angle: f64) -> Self {
		let radians = deg_to_rad(angle);
		let sin_theta = f64::sin(radians);
		let cos_theta = f64::cos(radians);

		let mut bbox = object.bounding_box();
		let mut min = Vec3::new(f64::MAX, f64::MAX, f64::MAX);
		let mut max = Vec3::new(f64::MIN, f64::MIN, f64::MIN);

		for i in 0..2 {
			for j in 0..2 {
				for k in 0..2 {
					let x = (i as f64) * bbox.x.max + ((1 - i) as f64) * bbox.x.min;
					let y = (j as f64) * bbox.y.max + ((1 - j) as f64) * bbox.y.min;
					let z = (k as f64) * bbox.z.max + ((1 - k) as f64) * bbox.z.min;

					let new_x = cos_theta * x + sin_theta * z;
					let new_z = -sin_theta * x + cos_theta * z;

					min.x = f64::min(min.x, new_x);
					max.x = f64::max(max.x, new_x);

					min.y = f64::min(min.y, y);
					max.y = f64::max(max.y, y);

					min.z = f64::min(min.z, new_z);
					max.z = f64::max(max.z, new_z);

				}
			}
		}

		bbox = AABB::from_corners(min, max);

		RotateY {
			object,
			sin_theta,
			cos_theta,
			bbox
		}
	}
}

impl Hittable for RotateY {
	fn hit(&self, ray: Ray, ray_t: Interval) -> Option<HitRecord> {
		let origin = Vec3::new(
			(self.cos_theta * ray.origin.x) - (self.sin_theta * ray.origin.z),
			ray.origin.y,
			(self.sin_theta * ray.origin.x) + (self.cos_theta * ray.origin.z)
		);

		let direction = Vec3::new(
			(self.cos_theta * ray.direction.x) - (self.sin_theta * ray.direction.z),
			ray.direction.y,
			(self.sin_theta * ray.direction.x) + (self.cos_theta * ray.direction.z)
		);

		let rotated_ray = Ray::new(origin, direction, ray.time);

		if let Some(mut hit_record) = self.object.hit(rotated_ray, ray_t) {
			hit_record.position = Vec3::new(
				(self.cos_theta * hit_record.position.x) + (self.sin_theta * hit_record.position.z),
				hit_record.position.y,
				(-self.sin_theta * hit_record.position.x) + (self.cos_theta * hit_record.position.z)
			);

			hit_record.normal = Vec3::new(
				(self.cos_theta * hit_record.normal.x) + (self.sin_theta * hit_record.normal.z),
				hit_record.normal.y,
				(-self.sin_theta * hit_record.normal.x) + (self.cos_theta * hit_record.normal.z)
			);

			return Some(hit_record);
		}

		None
	}

	fn bounding_box(&self) -> AABB {
		self.bbox
	}
}