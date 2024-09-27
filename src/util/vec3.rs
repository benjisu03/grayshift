use std::fmt;
use std::fmt::Formatter;
use std::ops::{Add, AddAssign, Div, DivAssign, Index, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Copy, Clone)]
pub struct Vec3 {
	pub x: f64,
	pub y: f64,
	pub z: f64
}

impl Vec3 {
	pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

	pub fn new(x: f64, y: f64, z: f64) -> Self {
		Vec3 { x, y, z }
	}

	pub fn length_squared(&self) -> f64 {
		self.x * self.x + self.y * self.y + self.z * self.z
	}

	pub fn length(&self) -> f64 {
		f64::sqrt(self.length_squared())
	}

	pub fn unit(&self) -> Vec3 {
		*self / self.length()
	}

	pub fn dot(&self, other: Vec3) -> f64 {
		self.x * other.x +
		self.y * other.y +
		self.z * other.z
	}

	pub fn cross(&self, other: Vec3) -> Vec3 {
		Vec3 {
			x: self.y * other.z - self.z * other.y,
			y: self.z * other.x - self.x * other.z,
			z: self.x * other.y - self.y * other.x
		}
	}

	const EPSILON: f64 = 1e-8;

	pub fn is_near_zero(&self) -> bool {
		(f64::abs(self.x) < Self::EPSILON) &&
		(f64::abs(self.y) < Self::EPSILON) &&
		(f64::abs(self.z) < Self::EPSILON)
	}

	pub fn reflect(&self, normal: Vec3) -> Vec3 {
		*self - 2.0 * self.dot(normal) * normal
	}

	pub fn refract(&self, normal: Vec3, refractive_ratio: f64) -> Vec3 {
		let cos_theta = f64::min(normal.dot(-(*self)), 1.0);
		let r_out_perp = refractive_ratio * (*self + cos_theta * normal);
		let r_out_parallel = -f64::sqrt(f64::abs(1.0 - r_out_perp.length_squared())) * normal;
		r_out_perp + r_out_parallel
	}
}

impl Neg for Vec3 {
	type Output = Self;
	fn neg(self) -> Self::Output {
		Vec3 {
			x: -self.x,
			y: -self.y,
			z: -self.z
		}
	}
}

impl Add for Vec3 {
	type Output = Self;
	fn add(self, rhs: Self) -> Self::Output {
		Vec3 {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
			z: self.z + rhs.z
		}
	}
}

impl AddAssign for Vec3 {
	fn add_assign(&mut self, rhs: Self) {
		self.x += rhs.x;
		self.y += rhs.y;
		self.z += rhs.z;
	}
}

impl Sub for Vec3 {
	type Output = Self;
	fn sub(self, rhs: Self) -> Self::Output {
		Vec3 {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
			z: self.z - rhs.z
		}
	}
}

impl SubAssign for Vec3 {
	fn sub_assign(&mut self, rhs: Self) {
		self.x -= rhs.x;
		self.y -= rhs.y;
		self.z -= rhs.z;
	}
}

impl Mul<f64> for Vec3 {
	type Output = Self;
	fn mul(self, rhs: f64) -> Self::Output {
		Vec3 {
			x: self.x * rhs,
			y: self.y * rhs,
			z: self.z * rhs
		}
	}
}

impl Mul<Vec3> for f64 {
	type Output = Vec3;
	fn mul(self, rhs: Vec3) -> Self::Output {
		rhs * self
	}
}

impl MulAssign<f64> for Vec3 {
	fn mul_assign(&mut self, rhs: f64) {
		self.x *= rhs;
		self.y *= rhs;
		self.z *= rhs;
	}
}

impl Mul<Vec3> for Vec3 {
	type Output = Vec3;

	fn mul(self, rhs: Vec3) -> Self::Output {
		Vec3 {
			x: self.x * rhs.x,
			y: self.y * rhs.y,
			z: self.z * rhs.z
		}
	}
}

impl Div<f64> for Vec3 {
	type Output = Self;
	fn div(self, rhs: f64) -> Self::Output {
		Vec3 {
			x: self.x / rhs,
			y: self.y / rhs,
			z: self.z / rhs
		}
	}
}

impl DivAssign<f64> for Vec3 {
	fn div_assign(&mut self, rhs: f64) {
		self.x /= rhs;
		self.y /= rhs;
		self.z /= rhs;
	}
}

impl fmt::Debug for Vec3 {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "Vec3({}, {}, {})", self.x, self.y, self.z)
	}
}

impl From<&[f32]> for Vec3 {
	fn from(v: &[f32]) -> Self {
		Vec3::new(v[0] as f64, v[1] as f64, v[2] as f64)
	}
}



