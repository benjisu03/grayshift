use nalgebra::Vector3;

pub fn random_f32(min: f32, max: f32) -> f32 {
	fastrand::f32() * (max - min) + min
}

pub fn random_vector(min: f32, max: f32) -> Vector3<f32> {
	Vector3::new(
		random_f32(min, max),
		random_f32(min, max),
		random_f32(min, max),
	)
}

pub fn random_vector_in_unit_sphere() -> Vector3<f32> {
	loop {
		let v = random_vector(-1.0, 1.0);
		if v.magnitude_squared() < 1.0 {
			return v;
		}
	}
}

pub fn random_unit_vector() -> Vector3<f32> {
	random_vector_in_unit_sphere().normalize()
}

pub fn random_vector_on_hemisphere(normal: Vector3<f32>) -> Vector3<f32> {
	let v = random_unit_vector();
	if v.dot(&normal) > 0.0 { v } else { -v }
}

pub fn random_vector_in_unit_disk() -> Vector3<f32> {
	loop {
		let v = Vector3::new(
			random_f32(-1.0, 1.0),
			random_f32(-1.0, 1.0),
			0.0,
		);

		if v.magnitude_squared() < 1.0 { return v; }
	}
}

pub fn random_cosine_direction() -> Vector3<f32> {
	let r1 = fastrand::f32();
	let r2 = fastrand::f32();

	let phi = 2.0 * std::f32::consts::PI * r1;

	let r2_sqrt = r2.sqrt();
	let (sin, cos) = phi.sin_cos();

	let x = cos * r2_sqrt;
	let y = sin * r2_sqrt;
	let z = (1.0 - r2).sqrt();

	Vector3::new(x, y, z)
}

pub fn rotate_vector(vector: Vector3<f32>, rotation: Vector3<f32>) -> Vector3<f32> {
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

	Vector3::new(x, y, z)
}

pub fn reflect(v: &Vector3<f32>, n: &Vector3<f32>) -> Vector3<f32> {
	v - 2.0 * v.dot(n) * n
}

pub fn refract(v: &Vector3<f32>, n: &Vector3<f32>, refractive_ratio: f32) -> Vector3<f32> {
	let cos_theta = (-*v).dot(n).min(1.0);
	let r_out_perp = refractive_ratio * (*v + cos_theta * *n);
	let r_out_parallel = -(1.0 - r_out_perp.magnitude_squared()).abs().sqrt() * *n;

	r_out_perp + r_out_parallel
}