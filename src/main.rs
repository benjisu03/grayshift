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

use std::error::Error;
use std::f64::consts::PI;
use crate::camera::{Background, Camera, SampleSettings, HDRI};
use crate::hittable::hittable::{Hittable, HittableList, RotateY, Translate};
use crate::material::{Dielectric, DiffuseLight, EmptyMaterial, Lambertian, Material, Metal};
use crate::hittable::sphere::Sphere;
use crate::util::util::{random_f64, random_vector};
use crate::util::vec3::Vec3;
use log::{info, LevelFilter};
use std::fs::File;
use std::io::{BufReader, Write};
use std::mem;
use std::sync::Arc;
use image::error::UnsupportedErrorKind::Format;
use wgpu::util::DeviceExt;
use crate::util::mesh::Mesh;
use crate::hittable::BVH::BVHNode;
use crate::hittable::quad::Quad;
use crate::hittable::triangle::Triangle;
use crate::texture::{CheckeredTexture, ImageTexture, NoiseTexture, SolidColorTexture};
use crate::hittable::volume::ConstantMedium;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {


	let mut logger = colog::default_builder();
	logger.filter_level(LevelFilter::Trace);
	logger.init();

	let mut image_file = File::create("image.ppm")?;

	meshes(&mut image_file).await?;
	Ok(())
}

async fn meshes(image_file: &mut File) -> Result<(), Box<dyn Error>> {

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


	let numbers = (0..10).collect::<Vec<i32>>();

	let instance = wgpu::Instance::default();
	let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await.unwrap();
	let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await.unwrap();

	let shader_module = device.create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));

	let buffer_size = (mem::size_of::<i32>() * numbers.len()) as wgpu::BufferAddress;

	let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
		label: Some("Input Buffer"),
		contents: bytemuck::cast_slice(&numbers),
		usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC
	});

	let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
		label: Some("Output Buffer"),
		size: buffer_size,
		usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
		mapped_at_creation: false,
	});

	let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
		label: Some("Compute Pipeline"),
		layout: None,
		module: &shader_module,
		entry_point: "main",
		compilation_options: Default::default(),
		cache: None
	});
	
	let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
	let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
		label: None,
		layout: &bind_group_layout,
		entries: &[wgpu::BindGroupEntry {
			binding: 0,
			resource: input_buffer.as_entire_binding(),
		}, wgpu::BindGroupEntry {
			binding: 1,
			resource: output_buffer.as_entire_binding(),
		}]
	});

	let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
	{
		let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
			label: None,
			timestamp_writes: None
		});
		cpass.set_pipeline(&compute_pipeline);
		cpass.set_bind_group(0, &bind_group, &[]);
		cpass.dispatch_workgroups(numbers.len() as u32, 1, 1);
	}

	let results_buffer = device.create_buffer(&wgpu::BufferDescriptor {
		label: Some("Results Buffer"),
		size: buffer_size,
		usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
		mapped_at_creation: false
	});

	encoder.copy_buffer_to_buffer(&output_buffer, 0, &results_buffer, 0, buffer_size);

	queue.submit(Some(encoder.finish()));

	let buffer_slice = results_buffer.slice(..);
	let (sender,  receiver) = flume::bounded(1);
	buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

	device.poll(wgpu::Maintain::wait()).panic_on_timeout();

	if let Ok(Ok(())) = receiver.recv_async().await {
		let data = buffer_slice.get_mapped_range();
		let results: Vec<i32> = bytemuck::cast_slice(&data).to_vec();

		drop(data);
		results_buffer.unmap();

		println!("{:?} => {:?}", numbers, results);

	} else {
		panic!("Failed to run compute");
	}


	let HDRI_file = File::open("airport.hdr")?;
	let HDRI_image = radiant::load(BufReader::new(HDRI_file))?;

	let camera_center = Vec3::new(-600.0, 300.0, 800.0);
	let camera_look_at = Vec3::new(0.0, 100.0, 0.0);
	let focus_distance = (camera_look_at - camera_center).length();

	let mut camera = Camera::new(
		16.0 / 9.0,
		600,
		SampleSettings {
			confidence: 0.95, // 95% confidence => 1.96
			tolerance: 0.05,
			batch_size: 32,
			max_samples: 100
		},
		2,
		20.0,
		camera_center,
		camera_look_at,
		Vec3::new(0.0, 1.0, 0.0),
		0.6,
		focus_distance,
		Background::HDRI(HDRI {
			image: HDRI_image,
			rotation: Vec3::new(PI / 2.0, PI, 0.0)
		})
	);

	let lights = Arc::new(Sphere::new_stationary(Vec3::ZERO, 1.0, metal));

	let world_bvh = BVHNode::from_list(world);
	camera.render(world_bvh, lights, image_file)?;

	Ok(())
}