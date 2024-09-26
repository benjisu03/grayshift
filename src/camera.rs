use crate::color::{luminance, write_color};
use crate::ray::Ray;
use indicatif::{ProgressBar, ProgressIterator};
use std::f64::consts::PI;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use log::info;
use crate::hittable::hittable::Hittable;
use crate::util::interval::Interval;
use crate::util::util::{deg_to_rad, random_vector_in_unit_disk};
use crate::util::vec3::Vec3;

use rayon::prelude::*;

pub struct Camera {
	image_width: i32,
	image_height: i32,

	sample_settings: SampleSettings,
	max_depth: u32,

	center: Vec3,
	starting_pixel_pos: Vec3,
	pixel_delta_u: Vec3,
	pixel_delta_v: Vec3,

	background: Vec3,

	defocus_angle: f64,
	defocus_disk_u: Vec3,
	defocus_disk_v: Vec3,
}

impl Camera {

	// PUBLIC //
	pub fn new(
		aspect_ratio: f64,
		image_width: i32,
		sample_settings: SampleSettings,
		max_depth: u32,
		v_fov: f64,
		look_from: Vec3,
		look_at: Vec3,
		vup: Vec3,
		defocus_angle: f64,
		focus_distance: f64,
		background: Vec3
	) -> Self {
		let image_height = (image_width as f64 / aspect_ratio) as i32;

		let theta = v_fov / 180.0 * PI;
		let h = f64::tan(theta / 2.0);
		let viewport_height = 2.0 * h * focus_distance;
		let viewport_width = viewport_height * (image_width as f64 / image_height as f64);

		let w = (look_from - look_at).unit();
		let u = vup.cross(w).unit();
		let v = w.cross(u);

		let viewport_u = viewport_width * u;
		let viewport_v = viewport_height * -v;

		let pixel_delta_u = viewport_u / image_width as f64;
		let pixel_delta_v = viewport_v / image_height as f64;

		let viewport_upper_left = look_from
			- focus_distance * w
			- viewport_u / 2.0
			- viewport_v / 2.0;
		let starting_pixel_pos = viewport_upper_left
			+ 0.5 * (pixel_delta_u + pixel_delta_v);

		let defocus_radius = focus_distance * f64::tan(deg_to_rad(defocus_angle / 2.0));
		let defocus_disk_u = u * defocus_radius;
		let defocus_disk_v = v * defocus_radius;

		Camera {
			image_width,
			image_height,

			sample_settings,
			max_depth,

			center: look_from,
			starting_pixel_pos,
			pixel_delta_u,
			pixel_delta_v,

			background,

			defocus_angle,
			defocus_disk_u,
			defocus_disk_v
		}
	}

	pub fn render(&self, world: Box<dyn Hittable>, image_file: &mut File) -> std::io::Result<()> {
		writeln!(image_file, "P3")?;
		writeln!(image_file, "{} {}", self.image_width, self.image_height)?;
		writeln!(image_file, "255")?;

		let pixels:Vec<i32> = (0..(self.image_width * self.image_height)).collect();
		let mut colors = Vec::with_capacity(pixels.len());

		let progress = Arc::new(ProgressBar::new(pixels.len() as u64));

		pixels.par_iter().map(|n| {
			let i = n % self.image_width;
			let j = n / self.image_width;
			self.sample(i, j, &world, progress.clone())
		}).collect_into_vec(&mut colors);

		for color in colors {
			write_color(image_file, color);
		}

		Ok(())
	}

	// PRIVATE //

	fn sample(&self, i: i32, j: i32, world: &Box<dyn Hittable>, progress: Arc<ProgressBar>) -> Vec3 {
		let mut pixel_color = Vec3::ZERO;

		let tolerance_sq = self.sample_settings.tolerance * self.sample_settings.tolerance;
		let confidence_sq = self.sample_settings.confidence * self.sample_settings.confidence;

		let mut sum = 0.0;
		let mut sq_sum = 0.0;
		let mut sample_count = 0.0;

		loop {
			// do a batch of samples
			sample_count += self.sample_settings.batch_size as f64;
			for _ in 0..self.sample_settings.batch_size {
				let ray = self.get_ray(i, j);
				let sample_color = self.ray_color(ray, self.max_depth, &world);
				pixel_color += sample_color;

				// luminance allows 1D tolerance based on human perception
				let lum = luminance(sample_color);
				sum += lum;
				sq_sum += lum * lum;
			}

			// calculate variance
			let mean = sum / sample_count;
			let variance_sq = 1.0 / (sample_count - 1.0) * (sq_sum - sum * sum / sample_count);

			let convergence_sq = confidence_sq * variance_sq / sample_count;

			// check if convergence is within tolerance, squared to reduce calculations
			if convergence_sq < (mean * mean * tolerance_sq) {
				break;
			}

			// some pixels take too long to converge
			// this means tolerance is not guaranteed, but adaptive sampling will at least speed up easy pixels
			if sample_count as u32 > self.sample_settings.max_samples {
				break;
			}
		}

		pixel_color /= sample_count;

		progress.inc(1);
		pixel_color
	}


	fn ray_color(&self, ray: Ray, depth: u32, world: &Box<dyn Hittable>) -> Vec3 {
		if depth <= 0 { return Vec3::ZERO }

		if let Some(hit_record) = world.hit(ray, Interval::new(0.001, f64::MAX)) {
			let emission_color = hit_record.material.emitted(
				hit_record.u,
				hit_record.v,
				hit_record.position
			);

			let material = hit_record.material.as_ref();
			if let Some(scatter_record) = material.scatter(ray, &hit_record) {

				let scatter_color = self.ray_color(
					scatter_record.scattered_ray,
					depth - 1,
					world
				);

				let color_from_scatter = scatter_color * scatter_record.attenuation;

				return emission_color + color_from_scatter;
			}

			return emission_color;
		}

		self.background
	}

	fn get_ray(&self, i: i32, j: i32) -> Ray {
		let offset_x = fastrand::f64() - 0.5;
		let offset_y = fastrand::f64() - 0.5;
		let pixel_sample = self.starting_pixel_pos
							   + (i as f64 + offset_x) * self.pixel_delta_u
							   + (j as f64 + offset_y) * self.pixel_delta_v;

		let ray_origin = if self.defocus_angle <= 0.0 {
			self.center
		} else {
			self.defocus_disk_sample()
		};

		let ray_direction = pixel_sample - ray_origin;
		let ray_time = fastrand::f64();

		Ray::new(ray_origin, ray_direction, ray_time)
	}

	fn defocus_disk_sample(&self) -> Vec3 {
		let v = random_vector_in_unit_disk();
		self.center + v.x * self.defocus_disk_u + v.y * self.defocus_disk_v
	}

}

pub struct SampleSettings {
	pub confidence: f64, // z-value
	pub tolerance: f64,
	pub batch_size: u32,
	pub max_samples: u32
}