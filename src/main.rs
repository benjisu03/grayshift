mod color;
mod ray;
mod hittable;
mod camera;
mod util;
mod material;
mod AABB;
mod texture;
mod ONB;
mod bsdf;
mod bsdf;

use std::error::Error;
use std::f64::consts::PI;
use crate::camera::{Background, Camera, SampleSettings, HDRI};
use crate::hittable::hittable::{Hittable, HittableList, RotateY, Translate};
use crate::material::{Dielectric, DiffuseLight, Lambertian, Material, Metal};
use crate::hittable::sphere::Sphere;
use crate::util::util::{random_f64, random_vector};
use crate::util::vec3::Vec3;
use log::{info, LevelFilter};
use std::fs::File;
use std::io::{BufReader, Write};
use std::sync::Arc;
use image::error::UnsupportedErrorKind::Format;
use crate::util::mesh::Mesh;
use crate::hittable::BVH::BVHNode;
use crate::hittable::quad::Quad;
use crate::hittable::triangle::Triangle;
use crate::texture::{CheckeredTexture, ImageTexture, NoiseTexture, SolidColorTexture};
use crate::hittable::volume::ConstantMedium;

fn main() -> Result<(), Box<dyn Error>> {


	let mut logger = colog::default_builder();
	logger.filter_level(LevelFilter::Trace);
	logger.init();

	let mut image_file = File::create("image.ppm")?;


	const SCENE: u8 = 12;

	match SCENE {
		12 => meshes(&mut image_file),
		11 => hdri(&mut image_file),
		// 10 => triangles(&mut image_file),
		9 => final_scene(&mut image_file, 800, 40),
		8 => final_scene(&mut image_file, 400, 50),
		7 => cornell_smoke(&mut image_file),
		6 => cornell_box(&mut image_file),
		5 => simple_light(&mut image_file),
		4 => quads(&mut image_file),
		3 => perlin_spheres(&mut image_file),
		2 => earth(&mut image_file),
		1 => checkered_spheres(&mut image_file),
		_ => bouncing_spheres(&mut image_file),
	}

}

// SCENES //

fn bouncing_spheres(image_file: &mut File) -> Result<(), Box<dyn Error>> {
	let mut world = HittableList::new();

	let ground_texture = Arc::new(CheckeredTexture::from_colors(
		0.32,
		Vec3::new(0.2, 0.3, 0.1),
		Vec3::new(0.9, 0.9, 0.9)
	));
	let ground_material = Arc::new(Lambertian::from_texture(ground_texture));
	world.add(Box::new(Sphere::new_stationary(
		Vec3::new(0.0, -1000.0, 0.0),
		1000.0,
		ground_material
	)));

	for a in -11..11 {
		for b in -11..11 {
			let center = Vec3::new(
				a as f64 + 0.9 * fastrand::f64(),
				0.2,
				b as f64 + 0.9 * fastrand::f64(),
			);

			if (center - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
				let material_choice = fastrand::f64();
				let material: Arc<dyn Material> = if material_choice < 0.8 {
					// diffuse
					Arc::new(Lambertian::from_color(
						random_vector(0.0, 1.0) * random_vector(0.0, 1.0))
					)

				} else if material_choice < 0.95 {
					// metal
					Arc::new(Metal::new(
						random_vector(0.5, 1.0),
						random_f64(0.0, 0.5)
					))

				} else {
					// glass

					Arc::new(Dielectric::new(1.5))
				};

				let center_end = center + Vec3::new(0.0, fastrand::f64(), 0.0);

				world.add(Box::new(Sphere::new_stationary(
					center,
					//center_end,
					0.2,
					material
				)));
			}
		}
	}

	let close_material = Arc::new(Metal::new(Vec3::new(0.7, 0.6, 0.5), 0.0));
	world.add(Box::new(Sphere::new_stationary(
		Vec3::new(4.0, 1.0, 0.0),
		1.0,
		close_material
	)));

	let far_material = Arc::new(Dielectric::new(1.5));
	world.add(Box::new(Sphere::new_stationary(
		Vec3::new(0.0, 1.0, 0.0),
		1.0,
		far_material
	)));

	let farther_material = Arc::new(Metal::new(Vec3::new(0.7, 0.6, 0.5), 0.0));
	world.add(Box::new(Sphere::new_stationary(
		Vec3::new(-4.0, 1.0, 0.0),
		1.0,
		farther_material
	)));

	let HDRI_file = File::open("airport.hdr")?;
	let HDRI_image = radiant::load(BufReader::new(HDRI_file))?;

	let mut camera = Camera::new(
		16.0 / 9.0,
		600,
		SampleSettings {
			confidence: 0.95, // 95% confidence => 1.96
			tolerance: 0.05,
			batch_size: 64,
			max_samples: 200
		},
		50,
		20.0,
		Vec3::new(13.0, 2.0, 3.0),
		Vec3::new(0.0, 0.0, 0.0),
		Vec3::new(0.0, 1.0, 0.0),
		0.6,
		10.0,
		Background::HDRI(HDRI {
			image: HDRI_image,
			rotation: Vec3::new(0.0, -90.0, 90.0)
		})
	);

	let world_bvh = BVHNode::from_list(world);
	camera.render(world_bvh, image_file)?;

	Ok(())
}

fn checkered_spheres(image_file: &mut File) -> Result<(), Box<dyn Error>> {

	let mut world = HittableList::new();

	let checkered_texture = Arc::new(CheckeredTexture::from_colors(
		0.32,
		Vec3::new(0.2, 0.3, 0.1),
		Vec3::new(0.9, 0.9, 0.9)
	));
	let checkered_material = Arc::new(Lambertian::from_texture(checkered_texture));

	world.add(Box::new(Sphere::new_stationary(
		Vec3::new(0.0, -10.0, 0.0),
		10.0,
		checkered_material.clone()
	)));

	world.add(Box::new(Sphere::new_stationary(
		Vec3::new(0.0, 10.0, 0.0),
		10.0,
		checkered_material.clone()
	)));

	let camera = Camera::new(
		16.0 / 9.0,
		400,
		SampleSettings {
			confidence: 0.95, // 95% confidence => 1.96
			tolerance: 0.25,
			batch_size: 32,
			max_samples: 1000
		},
		50,
		20.0,
		Vec3::new(13.0, 2.0, 3.0),
		Vec3::new(0.0, 0.0, 0.0),
		Vec3::new(0.0, 1.0, 0.0),
		0.0,
		10.0,
		Background::SOLID(Vec3::new(0.7, 0.8, 1.0))
	);

	let world_bvh = BVHNode::from_list(world);
	camera.render(world_bvh, image_file)?;

	Ok(())
}

fn earth(image_file: &mut File) -> Result<(), Box<dyn Error>> {

	let mut world = HittableList::new();

	let earth_texture = Arc::new(ImageTexture::new("earthmap.jpg")?);
	let earth_material = Arc::new(Lambertian::from_texture(earth_texture));

	world.add(Box::new(Sphere::new_stationary(
		Vec3::new(0.0, 0.0, 0.0),
		2.0,
		earth_material.clone()
	)));

	let camera = Camera::new(
		16.0 / 9.0,
		400,
		SampleSettings {
			confidence: 0.95, // 95% confidence => 1.96
			tolerance: 0.25,
			batch_size: 32,
			max_samples: 1000
		},
		50,
		20.0,
		Vec3::new(0.0, 0.0, 12.0),
		Vec3::new(0.0, 0.0, 0.0),
		Vec3::new(0.0, 1.0, 0.0),
		0.0,
		10.0,
		Background::SOLID(Vec3::new(0.7, 0.8, 1.0))
	);

	let world_bvh = BVHNode::from_list(world);
	camera.render(world_bvh, image_file)?;

	Ok(())
}

fn perlin_spheres(image_file: &mut File) -> Result<(), Box<dyn Error>> {

	let mut world = HittableList::new();

	let perlin_texture = Arc::new(NoiseTexture::new(4.0));
	let perlin_material = Arc::new(Lambertian::from_texture(perlin_texture));

	world.add(Box::new(Sphere::new_stationary(
		Vec3::new(0.0, -1000.0, 0.0),
		1000.0,
		perlin_material.clone()
	)));

	world.add(Box::new(Sphere::new_stationary(
		Vec3::new(0.0, 2.0, 0.0),
		2.0,
		perlin_material.clone()
	)));

	let camera = Camera::new(
		16.0 / 9.0,
		400,
		SampleSettings {
			confidence: 0.95, // 95% confidence => 1.96
			tolerance: 0.25,
			batch_size: 32,
			max_samples: 1000
		},
		50,
		20.0,
		Vec3::new(13.0,2.0, 3.0),
		Vec3::new(0.0, 0.0, 0.0),
		Vec3::new(0.0, 1.0, 0.0),
		0.0,
		10.0,
		Background::SOLID(Vec3::new(0.7, 0.8, 1.0))
	);

	let world_bvh = BVHNode::from_list(world);
	camera.render(world_bvh, image_file)?;

	Ok(())
}

fn quads(image_file: &mut File) -> Result<(), Box<dyn Error>> {

	let mut world = HittableList::new();

	// Materials
	let left_red = Arc::new(Lambertian::from_color(Vec3::new(1.0, 0.2, 0.2)));
	let back_green = Arc::new(Lambertian::from_color(Vec3::new(0.2, 1.0, 0.2)));
	let right_blue = Arc::new(Lambertian::from_color(Vec3::new(0.2, 0.2, 1.0)));
	let upper_orange = Arc::new(Lambertian::from_color(Vec3::new(1.0, 0.5, 0.0)));
	let lower_teal = Arc::new(Lambertian::from_color(Vec3::new(0.2, 0.8, 0.8)));

	world.add(Box::new(Quad::new(
		Vec3::new(-3.0, -2.0, 5.0),
		Vec3::new(0.0, 0.0, -4.0),
		Vec3::new(0.0, 4.0, 0.0),
		left_red
	)));
	world.add(Box::new(Quad::new(
		Vec3::new(-2.0, -2.0, 0.0),
		Vec3::new(4.0, 0.0, 0.0),
		Vec3::new(0.0, 4.0, 0.0),
		back_green.clone()
	)));
	world.add(Box::new(Quad::new(
		Vec3::new(3.0, -2.0, 1.0),
		Vec3::new(0.0, 0.0, 4.0),
		Vec3::new(0.0, 4.0, 0.0),
		right_blue
	)));
	world.add(Box::new(Quad::new(
		Vec3::new(-2.0, 3.0, 1.0),
		Vec3::new(4.0, 0.0, 0.0),
		Vec3::new(0.0, 0.0, 4.0),
		upper_orange
	)));
	world.add(Box::new(Quad::new(
		Vec3::new(-2.0, -3.0, 5.0),
		Vec3::new(4.0, 0.0, 0.0),
		Vec3::new(0.0, 0.0, -4.0),
		lower_teal
	)));

	let camera = Camera::new(
		1.0,
		400,
		SampleSettings {
			confidence: 0.95, // 95% confidence => 1.96
			tolerance: 0.25,
			batch_size: 32,
			max_samples: 1000
		},
		50,
		80.0,
		Vec3::new(0.0,0.0, 9.0),
		Vec3::new(0.0, 0.0, 0.0),
		Vec3::new(0.0, 1.0, 0.0),
		0.0,
		10.0,
		Background::SOLID(Vec3::new(0.7, 0.8, 1.0))
	);

	let world_bvh = BVHNode::from_list(world);
	camera.render(world_bvh, image_file)?;

	Ok(())
}

fn simple_light(image_file: &mut File) -> Result<(), Box<dyn Error>> {

	let mut world = HittableList::new();

	let noise_texture = Arc::new(NoiseTexture::new(4.0));
	let noise_material = Arc::new(Lambertian::from_texture(noise_texture));

	world.add(Box::new(Sphere::new_stationary(
		Vec3::new(0.0, -1000.0, 0.0),
		1000.0,
		noise_material.clone()
	)));

	world.add(Box::new(Sphere::new_stationary(
		Vec3::new(0.0, 2.0, 0.0),
		2.0,
		noise_material.clone()
	)));

	let light_material = Arc::new(DiffuseLight::from_color(
		Vec3::new(4.0, 4.0, 4.0)
	));

	world.add(Box::new(Quad::new(
		Vec3::new(3.0, 1.0, -2.0),
		Vec3::new(2.0, 0.0, 0.0),
		Vec3::new(0.0, 2.0, 0.0),
		light_material.clone()
	)));

	let camera = Camera::new(
		16.0 / 9.0,
		1000,
		SampleSettings {
			confidence: 0.95, // 95% confidence => 1.96
			tolerance: 0.25,
			batch_size: 32,
			max_samples: 1000
		},
		50,
		20.0,
		Vec3::new(26.0,3.0, 6.0),
		Vec3::new(0.0, 2.0, 0.0),
		Vec3::new(0.0, 1.0, 0.0),
		0.0,
		10.0,
		Background::SOLID(Vec3::new(0.0, 0.0, 0.0))
	);

	let world_bvh = BVHNode::from_list(world);
	camera.render(world_bvh, image_file)?;

	Ok(())
}

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

	let camera = Camera::new(
		1.0,
		600,
		SampleSettings {
			confidence: 0.95, // 95% confidence => 1.96
			tolerance: 0.5,
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
	camera.render(world_bvh, image_file)?;

	Ok(())
}

fn cornell_smoke(image_file: &mut File) -> Result<(), Box<dyn Error>> {

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
		Vec3::new(7.0, 7.0, 7.0)
	));

	world.add(Box::new(Quad::new(
		Vec3::new(113.0, 554.0, 127.0),
		Vec3::new(330.0, 0.0, 0.0),
		Vec3::new(0.0, 0.0, 305.0),
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
	let translated_box1 = Box::new(Translate::new(rotated_box1, Vec3::new(265.0, 0.0, 295.0)));
	world.add(Box::new(ConstantMedium::from_isotropic_color(
		translated_box1,
		0.01,
		Vec3::new(0.0, 0.0, 0.0)
	)));

	let box2: Box<dyn Hittable> = Box::new(Quad::cube(
		Vec3::ZERO,
		Vec3::new(165.0, 165.0, 165.0),
		white_material.clone()
	));
	let rotated_box2 = Box::new(RotateY::new(box2, -18.0));
	let translated_box2 = Box::new(Translate::new(rotated_box2, Vec3::new(130.0, 0.0, 65.0)));
	world.add(Box::new(ConstantMedium::from_isotropic_color(
		translated_box2,
		0.01,
		Vec3::new(1.0, 1.0, 1.0)
	)));


	let camera = Camera::new(
		1.0,
		600,
		SampleSettings {
			confidence: 0.95, // 95% confidence => 1.96
			tolerance: 0.25,
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
	camera.render(world_bvh, image_file)?;

	Ok(())
}

fn final_scene(
	image_file: &mut File,
	image_width: i32,
	max_depth: u32
) -> Result<(), Box<dyn Error>> {
	let mut world = HittableList::new();

	// GROUND BOXES //
	let ground_material = Arc::new(Lambertian::from_color(Vec3::new(0.48, 0.83, 0.53)));
	let boxes_per_side = 20;

	let mut boxes = HittableList::new();
	for i in 0..boxes_per_side {
		for j in 0..boxes_per_side {
			let w = 100.0;
			let x0 = -1000.0 + (i as f64) * w;
			let y0 = 0.0;
			let z0 = -1000.0 + (j as f64) * w;
			let x1 = x0 + w;
			let y1 = random_f64(1.0, 101.0);
			let z1 = z0 + w;

			boxes.add(Box::new(Quad::cube(
				Vec3::new(x0, y0, z0),
				Vec3::new(x1, y1, z1),
				ground_material.clone()
			)));
		}
	}

	world.add(BVHNode::from_list(boxes));

	// LIGHT //
	let light_material = Arc::new(DiffuseLight::from_color(Vec3::new(7.0, 7.0, 7.0)));
	world.add(Box::new(Quad::new(
		Vec3::new(123.0, 554.0, 147.0),
		Vec3::new(300.0, 0.0, 0.0),
		Vec3::new(0.0, 0.0, 265.0),
		light_material
	)));


	// MOVING SPHERE //
	let moving_center_1 = Vec3::new(400.0, 400.0, 200.0);
	let moving_center_2 = moving_center_1 + Vec3::new(30.0, 0.0, 0.0);
	let moving_material = Arc::new(Lambertian::from_color(Vec3::new(0.7, 0.3, 0.1)));
	world.add(Box::new(Sphere::new_moving(
		moving_center_1,
		moving_center_2,
		50.0,
		moving_material
	)));

	// GLASS SPHERE //
	let glass_material = Arc::new(Dielectric::new(1.5));
	world.add(Box::new(Sphere::new_stationary(
		Vec3::new(260.0, 150.0, 45.0),
		50.0,
		glass_material
	)));

	// METAL SPHERE //
	let metal_material = Arc::new(Metal::new(Vec3::new(0.8, 0.8, 0.9), 1.0));
	world.add(Box::new(Sphere::new_stationary(
		Vec3::new(0.0, 150.0, 145.0),
		50.0,
		metal_material
	)));

	// EARTH //
	let earth_texture = Arc::new(ImageTexture::new("earthmap.jpg")?);
	let earth_material = Arc::new(Lambertian::from_texture(earth_texture));
	world.add(Box::new(Sphere::new_stationary(
		Vec3::new(400.0, 200.0, 400.0),
		100.0,
		earth_material
	)));

	// NOISE SPHERE //
	let noise_texture = Arc::new(NoiseTexture::new(0.2));
	let noise_material = Arc::new(Lambertian::from_texture(noise_texture));
	world.add(Box::new(Sphere::new_stationary(
		Vec3::new(220.0, 280.0, 300.0),
		80.0,
		noise_material
	)));

	// FOGGY SPHERE //
	let fog_material = Arc::new(Dielectric::new(1.5));
	let foggy_sphere_boundary = Box::new(Sphere::new_stationary(
		Vec3::new(360.0, 150.0,145.0),
		70.0,
		fog_material.clone()
	));
	let foggy_sphere_material = Arc::new(Lambertian::from_color(Vec3::new(0.2, 0.4, 0.9)));
	world.add(Box::new(ConstantMedium::new(
		foggy_sphere_boundary,
		0.2,
		foggy_sphere_material
	)));

	// WORLD FOG //
	let world_fog_boundary = Box::new(Sphere::new_stationary(
		Vec3::ZERO,
		5000.0,
		fog_material
	));
	let world_fog_material = Arc::new(Lambertian::from_color(Vec3::new(1.0, 1.0, 1.0)));
	world.add(Box::new(ConstantMedium::new(
		world_fog_boundary,
		0.0001,
		world_fog_material
	)));

	// BALLS //
	let ball_material = Arc::new(Lambertian::from_color(Vec3::new(0.73, 0.73, 0.73)));
	let mut balls = HittableList::new();

	let ns = 1000;
	for i in 0..ns {
		balls.add(Box::new(Sphere::new_stationary(
			random_vector(0.0, 165.0),
			10.0,
			ball_material.clone()
		)));
	}

	world.add(
		Box::new(Translate::new(
			Box::new(RotateY::new(
				BVHNode::from_list(balls),
				15.0
			)),
			Vec3::new(-100.0, 270.0, 395.0)
		))
	);

	// CAMERA //
	let camera = Camera::new(
		1.0,
		image_width,
		SampleSettings {
			confidence: 0.95, // 95% confidence => 1.96
			tolerance: 0.25,
			batch_size: 32,
			max_samples: 1000
		},
		max_depth,
		40.0,
		Vec3::new(478.0, 278.0, -600.0),
		Vec3::new(278.0, 278.0, 0.0),
		Vec3::new(0.0, 1.0, 0.0),
		0.0,
		10.0,
		Background::SOLID(Vec3::new(0.0, 0.0, 0.0))
	);

	// RENDER //
	let world_bvh = BVHNode::from_list(world);
	camera.render(world_bvh, image_file)?;

	info!("Done.");
	Ok(())
}


// fn triangles(image_file: &mut File) -> Result<(), Box<dyn Error>> {
//
// 	let mut world = HittableList::new();
//
// 	let left_red = Arc::new(Lambertian::from_color(Vec3::new(1.0, 0.2, 0.2)));
// 	let back_green = Arc::new(Lambertian::from_color(Vec3::new(0.2, 1.0, 0.2)));
// 	let right_blue = Arc::new(Lambertian::from_color(Vec3::new(0.2, 0.2, 1.0)));
// 	let upper_orange = Arc::new(Lambertian::from_color(Vec3::new(1.0, 0.5, 0.0)));
// 	let lower_teal = Arc::new(Lambertian::from_color(Vec3::new(0.2, 0.8, 0.8)));
//
// 	world.add(Box::new(Triangle::new(
// 		Vec3::new(-2.0,  2.0, 0.0),
// 		Vec3::new(-2.0, -2.0, 0.0),
// 		Vec3::new(-2.0, -2.0, 4.0),
// 		left_red
// 	)));
// 	world.add(Box::new(Triangle::new(
// 		Vec3::new(-2.0, 2.0, 0.0),
// 		Vec3::new(2.0, -2.0, 0.0),
// 		Vec3::new(-2.0,-2.0, 0.0),
// 		back_green
// 	)));
// 	world.add(Box::new(Triangle::new(
// 		Vec3::new(-2.0, -2.0, 4.0),
// 		Vec3::new(-2.0, -2.0, 0.0),
// 		Vec3::new(2.0, -2.0, 0.0),
// 		upper_orange
// 	)));
//
// 	let camera = Camera::new(
// 		1.0,
// 		400,
// 		SampleSettings {
// 			confidence: 0.95, // 95% confidence => 1.96
// 			tolerance: 0.25,
// 			batch_size: 32,
// 			max_samples: 1000
// 		},
// 		50,
// 		80.0,
// 		Vec3::new(0.0,0.0, 9.0),
// 		Vec3::new(0.0, 0.0, 0.0),
// 		Vec3::new(0.0, 1.0, 0.0),
// 		0.0,
// 		10.0,
// 		Background::SOLID(Vec3::new(0.7, 0.8, 1.0))
// 	);
//
// 	let world_bvh = BVHNode::from_list(world);
// 	camera.render(world_bvh, image_file)?;
//
// 	Ok(())
// }

fn hdri(image_file: &mut File) -> Result<(), Box<dyn Error>> {

	let mut world = HittableList::new();

	// Materials

	let material = Arc::new(Metal::new(Vec3::new(0.7, 0.6, 0.5), 0.0));
	world.add(Box::new(Sphere::new_stationary(
		Vec3::new(4.0, 1.0, 0.0),
		1.0,
		material
	)));

	let HDRI_file = File::open("airport.hdr")?;
	let HDRI_image = radiant::load(BufReader::new(HDRI_file))?;

	let mut camera = Camera::new(
		16.0 / 9.0,
		600,
		SampleSettings {
			confidence: 0.95, // 95% confidence => 1.96
			tolerance: 0.05,
			batch_size: 64,
			max_samples: 200
		},
		50,
		20.0,
		Vec3::new(13.0, 2.0, 5.0),
		Vec3::new(0.0, 0.0, 0.0),
		Vec3::new(0.0, 1.0, 0.0),
		0.6,
		10.0,
		Background::HDRI(HDRI {
			image: HDRI_image,
			rotation: Vec3::new(PI / 2.0, PI, 0.0)
		})
	);

  let world_bvh = BVHNode::from_list(world);
	camera.render(world_bvh, image_file)?;

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

	let HDRI_file = File::open("airport.hdr")?;
	let HDRI_image = radiant::load(BufReader::new(HDRI_file))?;

	let camera_center = Vec3::new(-600.0, 300.0, 800.0);
	let camera_look_at = Vec3::new(0.0, 100.0, 0.0);
	let focus_dist = (camera_look_at - camera_center).length();

	let mut camera = Camera::new(
		16.0 / 9.0,
		600,
		SampleSettings {
			confidence: 0.95, // 95% confidence => 1.96
			tolerance: 0.001,
			batch_size: 32,
			max_samples: 10000
		},
		2,
		20.0,
		camera_center,
		camera_look_at,
		Vec3::new(0.0, 1.0, 0.0),
		0.6,
		focus_dist,
		Background::HDRI(HDRI {
			image: HDRI_image,
			rotation: Vec3::new(PI / 2.0, PI, 0.0)
		})
	);

	let world_bvh = BVHNode::from_list(world);
	camera.render(world_bvh, image_file)?;

	Ok(())
}