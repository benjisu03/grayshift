use std::f64::consts::PI;
use fastrand::f64;
use crate::util::vec3::Vec3;

pub fn random_f64(min: f64, max: f64) -> f64 {
	fastrand::f64() * (max - min) + min
}

pub fn random_vector(min: f64, max: f64) -> Vec3 {
	Vec3 {
		x: random_f64(min, max),
		y: random_f64(min, max),
		z: random_f64(min, max),

	}
}

pub fn random_vector_in_unit_sphere() -> Vec3 {
	loop {
		let v = random_vector(-1.0, 1.0);
		if v.length_squared() < 1.0 {
			return v;
		}
	}
}

pub fn random_unit_vector() -> Vec3 {
	random_vector_in_unit_sphere().unit()
}

pub fn random_vector_on_hemisphere(normal: Vec3) -> Vec3 {
	let v = random_unit_vector();
	if v.dot(normal) > 0.0 { v } else { -v }
}

pub fn random_vector_in_unit_disk() -> Vec3 {
	loop {
		let v = Vec3::new(
			random_f64(-1.0, 1.0),
			random_f64(-1.0, 1.0),
			0.0,
		);

		if v.length_squared() < 1.0 { return v; }
	}
}

pub fn random_cosine_direction() -> Vec3 {
	let r_1 = fastrand::f64();
	let r_2 = fastrand::f64();

	let phi = 2.0 * PI * r_1;
	let r_2_sqrt = f64::sqrt(r_2);

	let x = f64::cos(phi) * r_2_sqrt;
	let y = f64::sin(phi) * r_2_sqrt;
	let z = f64::sqrt(1.0 - r_2);

	Vec3 { x, y, z }
}

pub fn deg_to_rad(degrees: f64) -> f64 {
	degrees / 180.0 * PI
}


pub fn rotate_vector(vector: Vec3, rotation: Vec3) -> Vec3 {
	let (sin_x, cos_x) = rotation.x.sin_cos();
	let (sin_y, cos_y) = rotation.y.sin_cos();
	let (sin_z, cos_z) = rotation.z.sin_cos();

	// Combined rotation matrix
	let x = vector.x * (cos_y * cos_z) +
		vector.y * (cos_x * sin_z + sin_x * sin_y * cos_z) +
		vector.z * (sin_x * sin_z - cos_x * sin_y * cos_z);

	let y = vector.x * (-cos_y * sin_z) +
		vector.y * (cos_x * cos_z - sin_x * sin_y * sin_z) +
		vector.z * (sin_x * cos_z + cos_x * sin_y * sin_z);

	let z = vector.x * sin_y +
		vector.y * (-sin_x * cos_y) +
		vector.z * (cos_x * cos_y);

	Vec3::new(x, y, z)
}