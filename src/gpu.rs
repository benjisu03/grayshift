mod buffer;

use std::error::Error;
use std::sync::Arc;

use nalgebra::Vector3;
use wgpu::{BindGroupDescriptor, BindGroupEntry, BufferAddress, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, ComputePipelineDescriptor, DeviceDescriptor, include_wgsl, MapMode, RequestAdapterOptions};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::AABB::AABB;
use crate::hittable::BVH::BVH;
use crate::hittable::hittable::{HitRecord, Hittable};
use crate::hittable::triangle::Triangle;
use crate::material::Lambertian;
use crate::ray::Ray;
use crate::util::interval::Interval;

pub struct RayIntersectionManager {
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
	pub encoder: wgpu::CommandEncoder,

	pub compute_pipeline: wgpu::ComputePipeline,
	pub environment_bind_group: Option<wgpu::BindGroup>,
	pub ray_bind_group: Option<wgpu::BindGroup>,
	pub result_buffer: Option<wgpu::Buffer>
}

impl RayIntersectionManager {
	pub async fn new() -> Self {
		let instance = wgpu::Instance::default();
		let adapter = instance.request_adapter(&RequestAdapterOptions::default()).await.unwrap();

		let (device, queue) = adapter.request_device(&DeviceDescriptor::default(), None).await.unwrap();
		let encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });

		let shader_module = device.create_shader_module(include_wgsl!("shaders/intersection.wgsl"));
		let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
			label: Some("Ray Intersection Compute Pipeline"),
			layout: None,
			module: &shader_module,
			entry_point: "main",
			compilation_options: Default::default(),
			cache: None
		});


		Self {
			device,
			queue,
			encoder,

			compute_pipeline,
			environment_bind_group: None,
			ray_bind_group: None,
			result_buffer: None
		}
	}

	pub fn load_environment(&mut self, bvh: &BVH) {
		let (bvh_gpu, triangles_gpu) = bvh.to_gpu();

		let bvh_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
			label: Some("BVH Buffer"),
			contents: bytemuck::cast_slice(&bvh_gpu),
			usage: BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
		});

		let triangle_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Triangle Buffer"),
			contents: bytemuck::cast_slice(&triangles_gpu),
			usage: BufferUsages::STORAGE | BufferUsages::COPY_DST
		});

		let environment_bind_group_layout = self.compute_pipeline.get_bind_group_layout(0);
		self.environment_bind_group = Some(self.device.create_bind_group(&BindGroupDescriptor {
			label: Some("BVH Bind Group"),
			layout: &environment_bind_group_layout,
			entries: &[
				BindGroupEntry {
					binding: 0,
					resource: bvh_buffer.as_entire_binding(),
				},
				BindGroupEntry {
					binding: 1,
					resource: triangle_buffer.as_entire_binding(),
				}
			]
		}));

	}

	pub fn load_rays(&mut self, rays: Vec<Ray>) {
		let rays_gpu: Vec<RayGPU> = rays.iter().map(|r| { RayGPU::from(*r) }).collect();

		let ray_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Ray Buffer"),
			contents: bytemuck::cast_slice(&rays_gpu),
			usage: BufferUsages::STORAGE | BufferUsages::COPY_DST
		});

		let result_buffer = self.device.create_buffer(&BufferDescriptor {
			label: Some("Result Buffer"),
			size: (rays.len() * size_of::<TriangleIntersection>()) as BufferAddress,
			usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
			mapped_at_creation: false,
		});

		let ray_bind_group_layout = self.compute_pipeline.get_bind_group_layout(1);
		self.ray_bind_group = Some(self.device.create_bind_group(&BindGroupDescriptor {
			label: Some("Ray Bind Group"),
			layout: &ray_bind_group_layout,
			entries: &[
				BindGroupEntry {
					binding: 0,
					resource: ray_buffer.as_entire_binding(),
				},
				BindGroupEntry {
					binding: 1,
					resource: result_buffer.as_entire_binding(),
				},
			]
		}));

		self.result_buffer = Some(result_buffer);   
	}

	pub fn dispatch(&mut self) {

	}
}

pub async fn intersection_test(ray_manager: &RayIntersectionManager) -> Result<(), Box<dyn Error>> {

	let material = Arc::new(Lambertian::from_color(Vector3::new(0.5, 0.5, 0.5)));

	let center1 = Vector3::new(0.0, 0.0, 0.0);
	let center2 = Vector3::new(3.0, 0.0, 0.0);
	let center3 = Vector3::new(5.0, 0.0, 0.0);
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

	let rays = vec![r1, r2, r3];
	let rays_gpu: Vec<RayGPU> = rays.iter().map(|r| { RayGPU::from(*r) }).collect();

	let device = &ray_manager.device;

	let compute_pipeline = &ray_manager.compute_pipeline;

	let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
	{
		let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
			label: None,
			timestamp_writes: None
		});
		cpass.set_pipeline(&compute_pipeline);
		cpass.set_bind_group(0, ray_manager.environment_bind_group.as_ref().unwrap(), &[]);
		cpass.set_bind_group(1, ray_manager.ray_bind_group.as_ref().unwrap(), &[]);
		cpass.dispatch_workgroups(3, 1, 1);
	}

	let output_buffer = device.create_buffer(&BufferDescriptor {
		label: Some("Output Buffer"),
		size: 3 * size_of::<TriangleIntersection>() as BufferAddress,
		usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
		mapped_at_creation: false
	});

	encoder.copy_buffer_to_buffer(ray_manager.result_buffer.as_ref().unwrap(), 0, &output_buffer, 0, 3 * size_of::<TriangleIntersection>() as BufferAddress);
	ray_manager.queue.submit(Some(encoder.finish()));

	let buffer_slice = output_buffer.slice(..);
	let (sender, receiver) = flume::bounded(1);
	buffer_slice.map_async(MapMode::Read, move |v| { sender.send(v).unwrap() });

	device.poll(wgpu::Maintain::Wait).panic_on_timeout();

	if let Ok(Ok(..)) = receiver.recv_async().await {
		let data = buffer_slice.get_mapped_range();
		let results: Vec<TriangleIntersection> = bytemuck::cast_slice(&data).to_vec();

		drop(data);
		output_buffer.unmap();

		println!("{:?}", results);
	} else {
		panic!("Failed to run compute shader");
	}

	Ok(())
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct TriangleIntersection {
	pub id: u32,
	pub t: f32,
	pub u: f32,
	pub v: f32
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct RayGPU {
	pub origin: [f32; 3],
	_pad: f32,
	pub direction: [f32; 3],
	pub time: f32,
}

impl From<Ray> for RayGPU {
	fn from(value: Ray) -> Self {
		RayGPU {
			origin: [value.origin.x, value.origin.y, value.origin.z],
			direction: [value.direction.x, value.direction.y, value.direction.z],
			time: value.time,
			_pad: 0.0
		}
	}
}
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct AABBGPU {
	pub min: [f32; 3],
	_pad: f32,
	pub max: [f32; 3]
}

impl From<AABB> for AABBGPU {
	fn from(value: AABB) -> Self {
		AABBGPU {
			min: [value.x.min, value.y.min, value.z.min],
			max: [value.x.max, value.y.max, value.z.max],
			_pad: 0.0
		}
	}
}