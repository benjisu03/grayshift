use std::sync::Arc;
use crate::hittable::triangle::Triangle;
use crate::material::{Lambertian, Material};
use crate::util::util::random_vector;
use crate::util::vec3::Vec3;

pub struct Mesh {
    pub triangles: Vec<Triangle>,
}

pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3
}

impl Mesh {
    pub fn new(data: tobj::Mesh, material: Arc<dyn Material>) -> Self {
        let positions: Vec<Vec3> = data.positions.chunks(3).map(|chunk| { Vec3::from(chunk) }).collect();
        let normals: Vec<Vec3> = data.normals.chunks(3).map(|chunk| { Vec3::from(chunk) }).collect();

        let vertices: Vec<Vertex> = (0..data.indices.len()).map(|i| {
            let position_index = data.indices[i] as usize;
            let normal_index = data.normal_indices[i] as usize;

            let position = positions[position_index];
            let normal = normals[normal_index];

            Vertex { position, normal }
        }).collect();



        let triangles: Vec<Triangle> = vertices.chunks(3).map(|chunk| {
            let a = &chunk[0];
            let b = &chunk[1];
            let c = &chunk[2];

            let normal = a.normal;

            let color = Arc::new(Lambertian::from_color(random_vector(0.0, 1.0)));

            Triangle::new(a.position, b.position, c.position, normal, color)
        }).collect();

        Self { triangles }
    }
}