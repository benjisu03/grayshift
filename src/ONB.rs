use crate::util::vec3::Vec3;

pub struct OrthonormalBasis {
	pub u: Vec3,
	pub v: Vec3,
	pub w: Vec3
}

impl OrthonormalBasis {
	pub fn new(normal: Vec3) -> Self {
		let w = normal.unit();

		let a = if w.x.abs() > 0.9 {
			Vec3::new(0.0, 1.0, 0.0)
		} else {
			Vec3::new(1.0, 0.0, 0.0)
		};

		let v = w.cross(a).unit();
		let u = w.cross(v);

		OrthonormalBasis { u, v, w }
	}

	pub fn transform(&self, vec: Vec3) -> Vec3 {
		self.u * vec.x + self.v * vec.y + self.w * vec.z
	}
}