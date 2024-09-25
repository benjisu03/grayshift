use std::f64::consts::PI;
use std::fs::File;
use std::io::Write;
use indicatif::ProgressIterator;
use log::{info, trace};
use crate::color::write_color;
use crate::hittable::Hittable;
use crate::interval::Interval;
use crate::ray::Ray;
use crate::util::{deg_to_rad, random_vector_in_unit_disk, random_vector_on_hemisphere};
use crate::vec3::Vec3;

pub struct Camera {
	aspect_ratio: f64,
	image_width: i32,
	image_height: i32,
	samples_per_pixel: u32,
	pixel_samples_scale: f64,
	max_depth: u32,
	background: Vec3,
	defocus_angle: f64,
	defocus_disk_u: Vec3,
	defocus_disk_v: Vec3,
	center: Vec3,
	starting_pixel_pos: Vec3,
	pixel_delta_u: Vec3,
	pixel_delta_v: Vec3
}

impl Camera {

	// PUBLIC //
	pub fn new(aspect_ratio: f64,
	           image_width: i32,
	           samples_per_pixel: u32,
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

		let pixel_samples_scale = 1.0 / samples_per_pixel as f64;

		let defocus_radius = focus_distance * f64::tan(deg_to_rad(defocus_angle / 2.0));
		let defocus_disk_u = u * defocus_radius;
		let defocus_disk_v = v * defocus_radius;

		Camera {
			aspect_ratio,
			image_width,
			image_height,
			samples_per_pixel,
			pixel_samples_scale,
			max_depth,
			background,
			defocus_angle,
			defocus_disk_u,
			defocus_disk_v,
			center: look_from,
			starting_pixel_pos,
			pixel_delta_u,
			pixel_delta_v
		}
	}

	pub fn render(&self, world: Box<dyn Hittable>, image_file: &mut File) -> std::io::Result<()> {
		writeln!(image_file, "P3")?;
		writeln!(image_file, "{} {}", self.image_width, self.image_height)?;
		writeln!(image_file, "255")?;

		for j in (0..self.image_height).progress() {
			for i in 0..self.image_width {

				let mut pixel_color = Vec3::ZERO;
					for sample in 0..self.samples_per_pixel {
						let ray = self.get_ray(i, j);
						pixel_color += self.ray_color(ray, self.max_depth, &world);
					}

				write_color(image_file, pixel_color * self.pixel_samples_scale);
			}
		}

		Ok(())
	}

	// PRIVATE //

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
				let scattering_pdf = hit_record.material.scattering_pdf(ray, &hit_record, scatter_record.scattered_ray);
				let pdf_weight = scattering_pdf;

				let scatter_color = self.ray_color(
					scatter_record.scattered_ray,
					depth - 1,
					world
				);

				let color_from_scatter = scatter_color * scatter_record.attenuation * scattering_pdf / pdf_weight;

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