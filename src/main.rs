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
mod background;

use crate::camera::{Camera, SampleSettings};
use crate::hittable::hittable::{Hittable, HittableList, RotateY, Translate};
use crate::hittable::quad::Quad;
use crate::hittable::BVH::BVHNode;
use crate::material::{DiffuseLight, EmptyMaterial, Lambertian, Material, Metal};
use crate::util::mesh::Mesh;
use crate::util::vec3::Vec3;
use log::LevelFilter;
use std::error::Error;
use std::f64::consts::PI;
use std::fs::File;
use std::io::{BufReader, Write};
use std::sync::Arc;
use crate::background::{Background, CubeMap, HDRI};
use crate::texture::CheckeredTexture;

fn main() -> Result<(), Box<dyn Error>> {


	let mut logger = colog::default_builder();
	logger.filter_level(LevelFilter::Trace);
	logger.init();

	// Output
	let mut image_file = File::create("image.ppm")?;

	const SCENE: u8 = 2;

	match SCENE {
		2 => meshes(&mut image_file),
		_ => cornell_box(&mut image_file)
	}

}

// SCENES //
fn cornell_box(image_file: &mut File) -> Result<(), Box<dyn Error>> {

	let mut world = HittableList::new();

	let red_material = Arc::new(Lambertian::from_color(
		Vec3::new(0.65, 0.05, 0.05)
	));
	let white_material = Arc::new(Lambertian::from_color(
		Vec3::new(0.73, 0.73, 0.73)
	));
	let green_material = Arc::new(Lambertian::from_color(
		Vec3::new(0.12, 0.45, 0.15)
	));
	let light_material = Arc::new(DiffuseLight::from_color(
		Vec3::new(15.0, 15.0, 15.0)
	));

	world.add(Box::new(Quad::new(
		Vec3::new(343.0, 554.0, 332.0),
		Vec3::new(-130.0, 0.0, 0.0),
		Vec3::new(0.0, 0.0, -105.0),
		light_material.clone()
	)));

	world.add(Box::new(Quad::new(
		Vec3::new(555.0, 0.0, 0.0),
		Vec3::new(0.0, 555.0, 0.0),
		Vec3::new(0.0, 0.0, 555.0),
		green_material.clone()
	)));
	world.add(Box::new(Quad::new(
		Vec3::new(0.0, 0.0, 0.0),
		Vec3::new(0.0, 555.0, 0.0),
		Vec3::new(0.0, 0.0, 555.0),
		red_material.clone()
	)));
	world.add(Box::new(Quad::new(
		Vec3::new(0.0, 0.0, 0.0),
		Vec3::new(555.0, 0.0, 0.0),
		Vec3::new(0.0, 0.0, 555.0),
		white_material.clone()
	)));
	world.add(Box::new(Quad::new(
		Vec3::new(555.0, 555.0, 555.0),
		Vec3::new(-555.0, 0.0, 0.0),
		Vec3::new(0.0, 0.0, -555.0),
		white_material.clone()
	)));
	world.add(Box::new(Quad::new(
		Vec3::new(0.0, 0.0, 555.0),
		Vec3::new(555.0, 0.0, 0.0),
		Vec3::new(0.0, 555.0, 0.0),
		white_material.clone()
	)));

	let box1: Box<dyn Hittable> = Box::new(Quad::cube(
		Vec3::ZERO,
		Vec3::new(165.0, 330.0, 165.0),
		white_material.clone()
	));
	let rotated_box1 = Box::new(RotateY::new(box1, 15.0));
	let box1_final = Box::new(Translate::new(rotated_box1, Vec3::new(265.0, 0.0, 295.0)));
	world.add(box1_final);

	let box2: Box<dyn Hittable> = Box::new(Quad::cube(
		Vec3::ZERO,
		Vec3::new(165.0, 165.0, 165.0),
		white_material.clone()
	));
	let rotated_box2 = Box::new(RotateY::new(box2, -18.0));
	let box2_final = Box::new(Translate::new(rotated_box2, Vec3::new(130.0, 0.0, 65.0)));
	world.add(box2_final);

	let empty_material = Arc::new(EmptyMaterial::new());
	let lights = Arc::new(Quad::new(
		Vec3::new(343.0, 554.0, 332.0),
		Vec3::new(-130.0, 0.0, 0.0),
		Vec3::new(0.0, 0.0, -105.0),
		empty_material
	));

	let camera = Camera::new(
		1.0,
		600,
		SampleSettings {
			confidence: 0.95, // 95% confidence => 1.96
			tolerance: 0.001,
			batch_size: 32,
			max_samples: 1000
		},
		50,
		40.0,
		Vec3::new(278.0,278.0, -800.0),
		Vec3::new(278.0, 278.0, 0.0),
		Vec3::new(0.0, 1.0, 0.0),
		0.0,
		10.0,
		Background::SOLID(Vec3::new(0.0, 0.0, 0.0))
	);

	let world_bvh = BVHNode::from_list(world);
	camera.render(world_bvh, lights, image_file)?;

	Ok(())
}

fn meshes(image_file: &mut File) -> Result<(), Box<dyn Error>> {

	let mut world = HittableList::new();
	let metal = Arc::new(Metal::new(Vec3::new(0.7, 0.6, 0.5), 0.0));

	let load_options = tobj::LoadOptions {
		single_index: false,
		triangulate: false,
		ignore_points: false,
		ignore_lines: false,
	};
	let (models, materials) = tobj::load_obj("bmw/bmw.obj", &load_options)?;
	let materials = materials?;

	let lambertians: Vec<Arc<Lambertian>> = materials.iter().map(|material| {
		Arc::new(Lambertian::from_color(Vec3::from(material.diffuse.unwrap())))
	}).collect();

	for model in models {
		let material = &lambertians[model.mesh.material_id.unwrap()];

		let mesh = Mesh::new(model.mesh, material.clone());
		mesh.triangles.into_iter().for_each(|triangle| { world.add(Box::new(triangle))});
	}

	let light_material = Arc::new(DiffuseLight::from_color(
		Vec3::new(15.0, 15.0, 15.0)
	));

	world.add(Box::new(Quad::new(
		Vec3::new(200.0, 600.0, 200.0),
		Vec3::new(-400.0, 0.0, 0.0),
		Vec3::new(0.0, 0.0, -400.0),
		light_material.clone()
	)));

	let empty_material = Arc::new(EmptyMaterial::new());
	let lights = Arc::new(Quad::new(
		Vec3::new(200.0, 600.0, 200.0),
		Vec3::new(-400.0, 0.0, 0.0),
		Vec3::new(0.0, 0.0, -400.0),
		empty_material
	));

	let HDRI_file = File::open("metro.hdr")?;
	let HDRI_image = radiant::load(BufReader::new(HDRI_file))?;
	let camera_center = Vec3::new(-600.0, 300.0, 800.0);
	let camera_look_at = Vec3::new(0.0, 100.0, 0.0);
	let focus_distance = (camera_look_at - camera_center).length();

	let mut camera = Camera::new(
		16.0 / 9.0,
		800,
		SampleSettings {
			confidence: 0.95, // 95% confidence => 1.96
			tolerance: 0.05,
			batch_size: 32,
			max_samples: 20
		},
		2,
		20.0,
		camera_center,
		camera_look_at,
		Vec3::new(0.0, 1.0, 0.0),
		0.0,
		focus_distance,
		Background::SOLID(Vec3::new(0.0, 0.0, 0.0))
	);

	let world_bvh = BVHNode::from_list(world);
	camera.render(world_bvh, lights, image_file)?;

	Ok(())
}