use fastrand::f64;
use image::{DynamicImage, GenericImageView, ImageError};
use noise::{NoiseFn, Perlin};
use std::path::Path;
use std::rc::Rc;
use crate::util::vec3::Vec3;

pub trait Texture {
	fn value_at(&self, u: f64, v: f64, p: Vec3) -> Vec3;
}

pub struct SolidColorTexture {
	albedo: Vec3
}

impl SolidColorTexture {
	pub fn new(albedo: Vec3) -> Self {
		SolidColorTexture { albedo }
	}

	pub fn from_rgb(red: f64, green: f64, blue: f64) -> Self {
		SolidColorTexture { albedo: Vec3::new(red, green, blue) }
	}
}

impl Texture for SolidColorTexture {
	fn value_at(&self, u: f64, v: f64, p: Vec3) -> Vec3 {
		self.albedo
	}
}

pub struct CheckeredTexture {
	scale_inv: f64,
	even_texture: Rc<dyn Texture>,
	odd_texture: Rc<dyn Texture>
}

impl CheckeredTexture {
	pub fn new(scale: f64, even_texture: Rc<dyn Texture>, odd_texture: Rc<dyn Texture>) -> Self {
		CheckeredTexture {
			scale_inv: 1.0 / scale,
			even_texture,
			odd_texture
		}
	}

	pub fn from_colors(scale: f64, even_color: Vec3, odd_color: Vec3) -> Self {
		CheckeredTexture {
			scale_inv: 1.0 / scale,
			even_texture: Rc::new(SolidColorTexture::new(even_color)),
			odd_texture: Rc::new(SolidColorTexture::new(odd_color))
		}
	}
}

impl Texture for CheckeredTexture {
	fn value_at(&self, u: f64, v: f64, p: Vec3) -> Vec3 {
		let x_int = f64::floor(self.scale_inv * p.x) as i32;
		let y_int = f64::floor(self.scale_inv * p.y) as i32;
		let z_int = f64::floor(self.scale_inv * p.z) as i32;

		let is_even = (x_int + y_int + z_int) % 2 == 0;

		if is_even {
			self.even_texture.value_at(u, v, p)
		} else {
			self.odd_texture.value_at(u, v, p)
		}
	}
}

pub struct ImageTexture {
	image: DynamicImage
}

impl ImageTexture {
	pub fn new<P: AsRef<Path>>(filepath: P) -> Result<Self, ImageError> {
		Ok(ImageTexture { image: image::open(filepath)? })
	}
}

impl Texture for ImageTexture {
	fn value_at(&self, u: f64, v: f64, p: Vec3) -> Vec3 {

		let u_clamp = u.clamp(0.0, 1.0);
		let v_clamp = 1.0 - v.clamp(0.0, 1.0);

		let i = (u_clamp * self.image.width() as f64) as u32;
		let j = (v_clamp * self.image.height() as f64) as u32;
		let pixel = self.image.get_pixel(i, j);

		Vec3::new(pixel[0] as f64, pixel[1] as f64, pixel[2] as f64) * (1.0 / 255.0)
	}
}

pub struct NoiseTexture {
	scale: f64,
	noise: Perlin
}

impl NoiseTexture {
	pub fn new(scale: f64) -> Self {
		NoiseTexture { scale, noise: Perlin::default() }
	}

	pub fn turbulence(&self, p: Vec3, depth: u32) -> f64 {
		let mut accum = 0.0;
		let mut sample_point = p;
		let mut weight = 1.0;

		for i in 0..depth {
			accum += weight * self.noise.get([
				sample_point.x,
				sample_point.y,
				sample_point.z
			]);
			weight /= 2.0;
			sample_point *= 2.0;
		}

		accum.abs()
	}
}

impl Texture for NoiseTexture {
	fn value_at(&self, u: f64, v: f64, p: Vec3) -> Vec3 {
		Vec3::new(0.5, 0.5, 0.5)
			* (1.0 + (self.scale * p.z + 10.0 * self.turbulence(p, 7)).sin())
	}
}