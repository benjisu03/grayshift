use crate::hittable::hittable::{HitRecord, Hittable, HittableList};
use crate::hittable::triangle::Triangle;
use crate::hittable::BVH::BVH;
use crate::material::Lambertian;
use crate::ray::Ray;
use crate::util::interval::Interval;
use crate::AABB::AABB;
use nalgebra::Vector3;
use std::error::Error;
use std::sync::Arc;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BindGroupDescriptor, BindGroupEntry, BufferAddress, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, ComputePipelineDescriptor, MapMode};


pub async fn intersection_test() -> Result<(), Box<dyn Error>> {

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

    let mut objects = Vec::new();
    objects.push(t1);
    objects.push(t2);
    objects.push(t3);

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
        (center1 - camera_center).normalize(),
        0.0
    );



    let bvh = BVH::new(objects)?;

    let interval = Interval::new(0.0, f32::INFINITY);
    let results_test: Vec<Option<HitRecord>> = [r1, r2, r3].iter().map(|r| bvh.hit(*r, interval)).collect();

    let (bvh_gpu, triangles_gpu) = bvh.to_gpu();

    let rays = vec![r1, r2, r3];
    let rays_gpu: Vec<RayGPU> = rays.iter().map(|r| RayGPU::from(*r)).collect();

    let instance = wgpu::Instance::default();
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await.unwrap();
    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await.unwrap();

    let shader_module = device.create_shader_module(wgpu::include_wgsl!("shaders/intersection.wgsl"));

    let bvh_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("BVH Buffer"),
        contents: bytemuck::cast_slice(&bvh_gpu),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST
    });

    let triangle_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Triangle Buffer"),
        contents: bytemuck::cast_slice(&triangles_gpu),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST
    });

    let ray_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Ray Buffer"),
        contents: bytemuck::cast_slice(&rays_gpu),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST
    });

    let result_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("Result Buffer"),
        size: 3 * size_of::<TriangleIntersection>() as BufferAddress,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: None,
        module: &shader_module,
        entry_point: "main",
        compilation_options: Default::default(),
        cache: None
    });

    let bvh_bind_group_layout = compute_pipeline.get_bind_group_layout(0);
    let bvh_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("BVH Bind Group"),
        layout: &bvh_bind_group_layout,
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
    });

    let ray_bind_group_layout = compute_pipeline.get_bind_group_layout(1);
    let ray_bind_group = device.create_bind_group(&BindGroupDescriptor {
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
    });

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None
        });
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, &bvh_bind_group, &[]);
        cpass.set_bind_group(1, &ray_bind_group, &[]);
        cpass.dispatch_workgroups(3, 1, 1);
    }

    let output_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("Output Buffer"),
        size: 3 * size_of::<TriangleIntersection>() as BufferAddress,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false
    });

    encoder.copy_buffer_to_buffer(&result_buffer, 0, &output_buffer, 0, 3 * size_of::<TriangleIntersection>() as BufferAddress);

    queue.submit(Some(encoder.finish()));

    let buffer_slice = output_buffer.slice(..);
    let (sender, receiver) = flume::bounded(1);
    buffer_slice.map_async(MapMode::Read, move |v| { sender.send(v).unwrap() });

    device.poll(wgpu::Maintain::Wait).panic_on_timeout();

    if let Ok(Ok(..)) = receiver.recv_async().await {
        let data = buffer_slice.get_mapped_range();
        let results: Vec<TriangleIntersection> = bytemuck::cast_slice(&data).to_vec();

        drop(data);
n n         output_buffer.unmap();

        println!("{:?}", results);
    } else {
        panic!("Failed to run compute shader");
    }

    Ok(())
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct TriangleIntersection {
    id: u32,
    t: f32,
    u: f32,
    v: f32
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct RayGPU {
    pub origin: Vector3<f32>,
    pub direction: Vector3<f32>,
    pub time: f32
}

impl From<Ray> for RayGPU {
    fn from(value: Ray) -> Self {
        RayGPU {
            origin: value.origin,
            direction: value.direction,
            time: value.time
        }
    }
}
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct AABBGPU {
    pub x: IntervalGPU,
    pub y: IntervalGPU,
    pub z: IntervalGPU
}

impl From<AABB> for AABBGPU {
    fn from(value: AABB) -> Self {
        AABBGPU {
            x: IntervalGPU::from(value.x),
            y: IntervalGPU::from(value.y),
            z: IntervalGPU::from(value.z)
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct IntervalGPU {
    pub min: f32,
    pub max: f32
}

impl From<Interval> for IntervalGPU {
    fn from(value: Interval) -> Self {
        IntervalGPU {
            min: value.min,
            max: value.max
        }
    }
}