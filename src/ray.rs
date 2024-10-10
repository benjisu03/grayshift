use nalgebra::Vector3;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
	pub origin: Vector3<f32>,
	pub direction: Vector3<f32>,
	pub time: f32
}

impl Ray {
	pub fn new(origin: Vector3<f32>, direction: Vector3<f32>, time: f32) -> Self {
		Ray { origin, direction, time }
	}

	pub fn at(&self, t: f32) -> Vector3<f32> {
		self.origin + self.direction * t
	}
}