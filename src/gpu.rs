use std::error::Error;
use std::sync::Arc;
use wgpu::BufferUsages;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::hittable::BVH::BVH;
use crate::hittable::hittable::HittableList;
use crate::hittable::sphere::Sphere;
use crate::material::Lambertian;
use crate::ray::Ray;
use crate::util::vec3::Vec3;

pub async fn intersection_test() -> Result<(), Box<dyn Error>> {

    let material = Arc::new(Lambertian::from_color(Vec3::new(0.5, 0.5, 0.5)));

    let center1 = Vec3::new(0.0, 0.0, 0.0);
    let s1 = Box::new(Sphere::new_stationary(
        center1,
        1.0,
        material.clone()
    ));

    let center2 = Vec3::new(3.0, 0.0, 0.0);
    let s2 = Box::new(Sphere::new_stationary(
        center2,
        1.0,
        material.clone()
    ));

    let center3 = Vec3::new(5.0, 0.0, 0.0);
    let s3 = Box::new(Sphere::new_stationary(
        center3,
        1.0,
        material.clone()
    ));

    let mut objects = HittableList::new();
    objects.add(s1);
    objects.add(s2);
    objects.add(s3);

    let camera_center = Vec3::new(0.0, 0.0, 10.0);

    let r1 = Ray::new(
        camera_center,
        (center1 - camera_center).unit(),
        0.0
    );

    let r2 = Ray::new(
        camera_center,
        (center2 - camera_center).unit(),
        0.0
    );

    let r3 = Ray::new(
        camera_center,
        (center1 - camera_center).unit(),
        0.0
    );



    let bvh = BVH::new(objects)?;
    let bvh_gpu = bvh.to_gpu();

    let instance = wgpu::Instance::default();
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await.unwrap();
    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await.unwrap();

    let shader_module = device.create_shader_module(wgpu::include_wgsl!("shaders/intersection.wgsl"));

    let bvh_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("BVH Uniform Buffer"),
        contents: bytemuck::cast_slice(&bvh_gpu),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST
    });


    Ok(())
}