mod color;
mod ray;
mod hittable;
mod camera;
mod util;
mod material;
mod AABB;
mod texture;
mod ONB;
mod pdf;
mod gpu;
mod engine;
mod output;
mod background;
mod world;

use crate::background::HDRIBackground;
use crate::camera::Camera;
use crate::engine::{Engine, RenderSettings, SampleSettings};
use crate::hittable::hittable::{Hittable, HittableList};
use crate::hittable::sphere::Sphere;
use crate::hittable::BVH::BVH;
use crate::material::{Lambertian, Material, Metal};
use crate::output::{PPMImage, RenderTarget};
use crate::util::mesh::Mesh;
use crate::world::World;
use log::LevelFilter;
use nalgebra::Vector3;
use std::error::Error;
use std::f32::consts::PI;
use std::io::Write;
use std::mem;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use crate::gpu::{intersection_test, RayIntersectionManager};
use crate::hittable::triangle::Triangle;
use crate::ray::Ray;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {


	let mut logger = colog::default_builder();
	logger.filter_level(LevelFilter::Error);
	logger.init();

	let image_width = 600;
	let aspect_ratio = 16.0 / 9.0;
	let image_height = (image_width as f64 / aspect_ratio) as u32;

	let image = Box::new(PPMImage::new("image.ppm", image_width, image_height)?);

	meshes(image).await?;
	Ok(())
}

async fn meshes(render_target: Box<dyn RenderTarget>) -> Result<(), Box<dyn Error>> {

	let mut objects = HittableList::new();
	let metal = Arc::new(Metal::new(Vector3::new(0.7, 0.6, 0.5), 0.0));

	let load_options = tobj::LoadOptions {
		single_index: false,
		triangulate: false,
		ignore_points: false,
		ignore_lines: false,
	};
	let (models, materials) = tobj::load_obj("bmw/bmw.obj", &load_options)?;
	let materials = materials?;

	let lambertians: Vec<Arc<Lambertian>> = materials.iter().map(|material| {
		Arc::new(Lambertian::from_color(Vector3::from(material.diffuse.unwrap())))
	}).collect();

	for model in models {
		let material = &lambertians[model.mesh.material_id.unwrap()];

		let mesh = Mesh::new(model.mesh, material.clone());
		mesh.triangles.into_iter().for_each(|triangle| { objects.add(Box::new(triangle))});
	}

	let background = Box::new(HDRIBackground::new(
		"airport.hdr",
		Vector3::new(PI / 2.0, PI, 0.0)
	)?);


	let camera_center = Vector3::new(-600.0, 300.0, 800.0);
	let camera_look_at = Vector3::new(0.0, 100.0, 0.0);
	let focus_distance = (camera_look_at - camera_center).magnitude();

	let render_settings = RenderSettings {
		sample_settings: SampleSettings {
			confidence: 0.95, // 95% confidence => 1.96
			tolerance: 0.05,
			batch_size: 32,
			max_samples: 100
		},
		max_ray_depth: 2,
	};

	let camera = Camera::new(
		render_target.size(),
		20.0,
		camera_center,
		camera_look_at,
		Vector3::new(0.0, 1.0, 0.0),
		0.6,
		focus_distance
	);

	let mut engine = Engine::new(
		camera,
		render_target,
		background,
		render_settings
	);

	let lights = Arc::new(Sphere::new_stationary(Vector3::new(0.0, 0.0, 0.0), 1.0, metal));
	// let objects_bvh = BVH::new(objects)?;
	// let world = World {
	// 	objects: Box::new(objects_bvh),
	// 	lights
	// };

	//engine.render(world)?;

	let material = Arc::new(Lambertian::from_color(Vector3::new(0.5, 0.5, 0.5)));

	let center1 = Vector3::new(0.0, 0.0, 0.0);
	let t1 = Box::new(Triangle::new(
		center1 + Vector3::new(-1.0, -1.0, 0.0),
		center1 + Vector3::new(0.0, 1.0, 0.0),
		center1 + Vector3::new(1.0, -1.0, 0.0),
		Vector3::new(0.0, 0.0, 1.0),
		material.clone()
	));

	let center2 = Vector3::new(3.0, 0.0, 0.0);
	let t2 = Box::new(Triangle::new(
		center2 + Vector3::new(-1.0, -1.0, 0.0),
		center2 + Vector3::new(0.0, 1.0, 0.0),
		center2 + Vector3::new(1.0, -1.0, 0.0),
		Vector3::new(0.0, 0.0, 1.0),
		material.clone()
	));

	let center3 = Vector3::new(5.0, 0.0, 0.0);
	let t3 = Box::new(Triangle::new(
		center3 + Vector3::new(-1.0, -1.0, 0.0),
		center3 + Vector3::new(0.0, 1.0, 0.0),
		center3 + Vector3::new(1.0, -1.0, 0.0),
		Vector3::new(0.0, 0.0, 1.0),
		material.clone()
	));

	let camera_center = Vector3::new(0.0, 0.0, 10.0);

	let r1 = Ray::new(
		camera_center,
		(center1 - camera_center).normalize(),
		0.0
	);

	let r2 = Ray::new(
		camera_center,
		(center2 - camera_center).normalize(),
		0.0
	);

	let r3 = Ray::new(
		camera_center,
		(center3 - camera_center).normalize(),
		0.0
	);

	let mut objects = Vec::new();
	objects.push(t1);
	objects.push(t2);
	objects.push(t3);

	let bvh = BVH::new(objects)?;

	let mut ray_manager = RayIntersectionManager::new().await;
	ray_manager.load_environment(&bvh);
	ray_manager.load_rays(vec![r1, r2, r3]);

	intersection_test(&ray_manager).await?;

	Ok(())
}