use crate::util::vec3::Vec3;

#[derive(Debug, Copy, Clone)]
pub struct Ray {
	pub origin: Vec3,
	pub direction: Vec3,
	pub time: f64
}

impl Ray {
	pub fn new(origin: Vec3, direction: Vec3, time: f64) -> Self {
		Ray { origin, direction, time }
	}

	pub fn at(&self, t: f64) -> Vec3 {
		self.origin + self.direction * t
	}
}