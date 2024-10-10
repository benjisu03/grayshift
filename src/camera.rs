use nalgebra::Vector3;
use crate::ray::Ray;
use crate::util::util::random_vector_in_unit_disk;

pub struct Camera {
	center: Vector3<f32>,

	starting_pixel_pos: Vector3<f32>,
	pixel_delta_u: Vector3<f32>,
	pixel_delta_v: Vector3<f32>,

	defocus_disk_u: Vector3<f32>,
	defocus_disk_v: Vector3<f32>
}

impl Camera {

	pub fn new(
		render_target_size: (u32, u32),
		v_fov: f32,
		look_from: Vector3<f32>,
		look_at: Vector3<f32>,
		vup: Vector3<f32>,
		defocus_angle: f32,
		focus_distance: f32
	) -> Self {

		let h = (v_fov.to_radians() / 2.0).tan();
		let viewport_height = 2.0 * h * focus_distance;
		let viewport_width = viewport_height * (render_target_size.0 as f32 / render_target_size.1 as f32);

		let w = (look_from - look_at).normalize();
		let u = vup.cross(&w).normalize();
		let v = w.cross(&u);

		let viewport_u = viewport_width * u;
		let viewport_v = viewport_height * -v;

		let pixel_delta_u = viewport_u / render_target_size.0 as f32;
		let pixel_delta_v = viewport_v / render_target_size.1 as f32;

		let viewport_upper_left = look_from - focus_distance * w - viewport_u / 2.0 - viewport_v / 2.0;
		let starting_pixel_pos = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

		let defocus_radius = focus_distance * (defocus_angle.to_radians() / 2.0).tan();
		let defocus_disk_u = u * defocus_radius;
		let defocus_disk_v = v * defocus_radius;

		let center = look_from;

		Camera { center, starting_pixel_pos, pixel_delta_u, pixel_delta_v, defocus_disk_u, defocus_disk_v }
	}

	pub fn get_ray(&self, pixel: (u32, u32), inner_offset: Vector3<f32>) -> Ray {
		let (i, j) = pixel;
		let pixel_sample = self.starting_pixel_pos
			+ self.pixel_delta_u * (i as f32 + inner_offset.x)
			+ self.pixel_delta_v * (j as f32 + inner_offset.y);

		let ray_origin = self.defocus_disk_sample();
		let ray_direction = pixel_sample - ray_origin;
		let ray_time = fastrand::f32();

		Ray::new(ray_origin, ray_direction, ray_time)
	}

	fn defocus_disk_sample(&self) -> Vector3<f32> {
		let v = random_vector_in_unit_disk();
		self.center + v.x * self.defocus_disk_u + v.y * self.defocus_disk_v
	}
}

