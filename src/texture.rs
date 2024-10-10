use crate::util::vec3::Vec3;
use image::{DynamicImage, GenericImageView, ImageError};
use nalgebra::Vector3;
use noise::{NoiseFn, Perlin};
use std::path::Path;
use std::sync::Arc;

pub trait Texture: Send + Sync {
	fn value_at(&self, u: f32, v: f32, p: Vector3<f32>) -> Vector3<f32>;
}

pub struct SolidColorTexture {
	albedo: Vector3<f32>
}

impl SolidColorTexture {
	pub fn new(albedo: Vector3<f32>) -> Self {
		SolidColorTexture { albedo }
	}

	pub fn from_rgb(red: f32, green: f32, blue: f32) -> Self {
		SolidColorTexture { albedo: Vector3::new(red, green, blue) }
	}
}

impl Texture for SolidColorTexture {
	fn value_at(&self, u: f32, v: f32, p: Vector3<f32>) -> Vector3<f32> {
		self.albedo
	}
}

pub struct CheckeredTexture {
	scale_inv: f32,
	even_texture: Arc<dyn Texture>,
	odd_texture: Arc<dyn Texture>
}

impl CheckeredTexture {
	pub fn new(scale: f32, even_texture: Arc<dyn Texture>, odd_texture: Arc<dyn Texture>) -> Self {
		CheckeredTexture {
			scale_inv: 1.0 / scale,
			even_texture,
			odd_texture
		}
	}

	pub fn from_colors(scale: f32, even_color: Vector3<f32>, odd_color: Vector3<f32>) -> Self {
		CheckeredTexture {
			scale_inv: 1.0 / scale,
			even_texture: Arc::new(SolidColorTexture::new(even_color)),
			odd_texture: Arc::new(SolidColorTexture::new(odd_color))
		}
	}
}

impl Texture for CheckeredTexture {
	fn value_at(&self, u: f32, v: f32, p: Vector3<f32>) -> Vector3<f32> {
		let x_int = f32::floor(self.scale_inv * p.x) as i32;
		let y_int = f32::floor(self.scale_inv * p.y) as i32;
		let z_int = f32::floor(self.scale_inv * p.z) as i32;

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
	fn value_at(&self, u: f32, v: f32, _p: Vector3<f32>) -> Vector3<f32> {

		let u_clamp = u.clamp(0.0, 1.0);
		let v_clamp = 1.0 - v.clamp(0.0, 1.0);

		let i = (u_clamp * self.image.width() as f32) as u32;
		let j = (v_clamp * self.image.height() as f32) as u32;
		let pixel = self.image.get_pixel(i, j);

		Vector3::new(pixel[0] as f32, pixel[1] as f32, pixel[2] as f32) * (1.0 / 255.0)
	}
}

pub struct NoiseTexture {
	scale: f32,
	noise: Perlin
}

impl NoiseTexture {
	pub fn new(scale: f32) -> Self {
		NoiseTexture { scale, noise: Perlin::default() }
	}

	pub fn turbulence(&self, p: Vector3<f32>, depth: u32) -> f32 {
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

		accum.abs() as f32
	}
}

impl Texture for NoiseTexture {
	fn value_at(&self, u: f32, v: f32, p: Vector3<f32>) -> Vector3<f32> {
		Vector3::new(0.5, 0.5, 0.5)
			* (1.0 + (self.scale * p.z + 10.0 * self.turbulence(p, 7)).sin())
	}
}