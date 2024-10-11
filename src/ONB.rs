use nalgebra::Vector3;

pub struct OrthonormalBasis {
	pub u: Vector3<f32>,
	pub v: Vector3<f32>,
	pub w: Vector3<f32>
}

impl OrthonormalBasis {
	pub fn new(normal: Vector3<f32>) -> Self {
		let w = normal.normalize();

		let a = if w.x.abs() > 0.9 {
			Vector3::new(0.0, 1.0, 0.0)
		} else {
			Vector3::new(1.0, 0.0, 0.0)
		};

		let v = w.cross(&a).normalize();
		let u = w.cross(&v);

		OrthonormalBasis { u, v, w }
	}

	pub fn transform(&self, vec: Vector3<f32>) -> Vector3<f32> {
		self.u * vec.x + self.v * vec.y + self.w * vec.z
	}
}