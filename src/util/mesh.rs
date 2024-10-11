use crate::hittable::triangle::Triangle;
use crate::material::Material;
use nalgebra::Vector3;
use std::sync::Arc;

pub struct Mesh {
    pub triangles: Vec<Triangle>,
}

pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>
}

impl Mesh {
    pub fn new(data: tobj::Mesh, material: Arc<dyn Material>) -> Self {
        let positions: Vec<Vector3<f32>> = data.positions.chunks(3).map(|chunk| { Vector3::new(chunk[0], chunk[1], chunk[2]) }).collect();
        let normals: Vec<Vector3<f32>> = data.normals.chunks(3).map(|chunk| { Vector3::new(chunk[0], chunk[1], chunk[2]) }).collect();

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

            let normal = a.normal.normalize();

            Triangle::new(a.position, b.position, c.position, normal, material.clone())
        }).collect();

        Self { triangles }
    }
}