use std::error::Error;
use crate::color::{luminance, write_color};
use crate::ray::Ray;
use indicatif::{ProgressBar, ProgressIterator};
use std::f64::consts::PI;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use log::info;
use crate::hittable::hittable::Hittable;
use crate::util::interval::Interval;
use crate::util::util::{deg_to_rad, random_vector_in_unit_disk, rotate_vector};
use crate::util::vec3::Vec3;

use rayon::prelude::*;
use crate::background::Background;
use crate::output::RenderTarget;

pub struct Camera {
	center: Vec3,
	starting_pixel_pos: Vec3,
	pixel_delta_u: Vec3,

	pixel_delta_v: Vec3,

	defocus_angle: f64,
	defocus_disk_u: Vec3,
	defocus_disk_v: Vec3,
}

impl Camera {

	// PUBLIC //
	pub fn new(
		render_target_size: (u32, u32),
		v_fov: f64,
		look_from: Vec3,
		look_at: Vec3,
		vup: Vec3,
		defocus_angle: f64,
		focus_distance: f64
	) -> Self {
		let (image_width, image_height) = render_target_size;

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
			center: look_from,
			starting_pixel_pos,
			pixel_delta_u,
			pixel_delta_v,
			defocus_angle,
			defocus_disk_u,
			defocus_disk_v
		}
	}

	pub fn get_ray(&self, offset: Vec3, i: u32, j: u32) -> Ray {
		let pixel_sample = self.starting_pixel_pos
							   + (i as f64 + offset.x) * self.pixel_delta_u
							   + (j as f64 + offset.y) * self.pixel_delta_v;

		let ray_origin = if self.defocus_angle <= 0.0 {
			self.center
		} else {
			self.defocus_disk_sample()
		};

		let ray_direction = pixel_sample - ray_origin;
		let ray_time = fastrand::f64();

		Ray::new(ray_origin, ray_direction, ray_time)
	}

	// PRIVATE //

	fn defocus_disk_sample(&self) -> Vec3 {
		let v = random_vector_in_unit_disk();
		self.center + v.x * self.defocus_disk_u + v.y * self.defocus_disk_v
	}
}

