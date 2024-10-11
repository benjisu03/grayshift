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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {


	let mut logger = colog::default_builder();
	logger.filter_level(LevelFilter::Trace);
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
	let objects_bvh = BVH::new(objects)?;
	let world = World {
		objects: Box::new(objects_bvh),
		lights
	};

	engine.render(world)?;

	//intersection_test().await?;

	Ok(())
}