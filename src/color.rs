use std::fs::File;
use std::io::Write;
use crate::util::interval::Interval;
use crate::util::vec3::Vec3;

const INTENSITY: Interval = Interval { min: 0.000, max: 0.999 };

pub fn write_color(image: &mut File, color: Vec3) {
	let r = linear_to_gamma(color.x);
	let g = linear_to_gamma(color.y);
	let b = linear_to_gamma(color.z);

	let r_byte = (256.0 * INTENSITY.clamp(r)) as i32;
	let g_byte = (256.0 * INTENSITY.clamp(g)) as i32;
	let b_byte = (256.0 * INTENSITY.clamp(b)) as i32;

	writeln!(image, "{r_byte} {g_byte} {b_byte}").unwrap();
}

fn linear_to_gamma(n: f64) -> f64 {
	if n > 0.0 {
		return f64::sqrt(n);
	}

	0.0
}