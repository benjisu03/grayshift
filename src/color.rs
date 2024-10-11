use std::fs::File;
use std::io::Write;
use nalgebra::Vector3;
use crate::util::interval::Interval;

const INTENSITY: Interval = Interval { min: 0.000, max: 0.999 };

pub fn write_color(image: &mut File, color: Vector3<f32>) -> std::io::Result<()> {
	let r = linear_to_gamma(color.x);
	let g = linear_to_gamma(color.y);
	let b = linear_to_gamma(color.z);

	let r_byte = (256.0 * INTENSITY.clamp(r)) as i32;
	let g_byte = (256.0 * INTENSITY.clamp(g)) as i32;
	let b_byte = (256.0 * INTENSITY.clamp(b)) as i32;

	writeln!(image, "{r_byte} {g_byte} {b_byte}")?;
	Ok(())
}

fn linear_to_gamma(n: f32) -> f32 {
	if n > 0.0 {
		return n.sqrt();
	}

	0.0
}


// Converts RGB into human-perceived luminance
// Formula source: https://www.w3.org/TR/AERT/#color-contrast
pub fn luminance(v: Vector3<f32>) -> f32 {
	0.299 * v.x + 0.587 * v.y + 0.144 * v.z
}