use crate::hittable::hittable::{HitRecord, Hittable, HittableList};
use crate::ray::Ray;
use crate::util::interval::Interval;
use crate::AABB::AABB;
use std::error::Error;
use nalgebra::Vector3;
use crate::gpu::AABBGPU;
use crate::hittable::triangle::{Triangle, TriangleGPU};

pub struct BVH {
	root: Box<BVHNode>,
}

enum BVHNode {
	Leaf(Box<Triangle>),
	Branch {
		left: Box<BVHNode>,
		right: Option<Box<BVHNode>>,
		bbox: AABB
	}
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
 pub struct BVHNodeGPU {
	bbox: AABBGPU,
	left: u32,
	right: u32,
	triangle_id: u32
}

impl BVH {
	pub fn new(list: Vec<Box<Triangle>>) -> Result<Self, Box<dyn Error>> {
		if list.is_empty() {
			return Err(Box::from("BVH cannot be built from empty list"));
		}

		let root = BVH::build(list);
		Ok(BVH { root })
	}

	fn build(mut triangles: Vec<Box<Triangle>>) -> Box<BVHNode> {

		if triangles.len() == 1 {
			let obj = triangles.swap_remove(0);
			return Box::new(BVHNode::Leaf(obj));
		}

		let mut bbox = AABB::EMPTY;
		for object in triangles.iter() {
			bbox = AABB::from_AABB_pair(bbox, object.bounding_box());
		}

		let axis = bbox.longest_axis();

		triangles.sort_by(|a, b| {
			let a_min = a.bounding_box()[axis].min;
			let b_min = b.bounding_box()[axis].min;
			a_min.partial_cmp(&b_min).unwrap()
		});

		let middle = triangles.len() / 2;
		let right_objs = triangles.split_off(middle);
		let left_objs = triangles;

		let left = BVH::build(left_objs);
		let right = Some(BVH::build(right_objs));

		Box::new(BVHNode::Branch { left, right, bbox })
	}

	pub fn to_gpu(&self) -> (Vec<BVHNodeGPU>, Vec<TriangleGPU>) {
		let mut flattened = Vec::new();
		let mut triangles = Vec::new();

		BVH::flatten(self.root.as_ref(), &mut flattened, &mut triangles);
		(flattened, triangles)
	}

	fn flatten(node: &BVHNode, flattened: &mut Vec<BVHNodeGPU>, triangles: &mut Vec<TriangleGPU>) -> u32 {
		match node {
			BVHNode::Leaf(triangle) => {
				let triangle_index = triangles.len() as u32;
				triangles.push(TriangleGPU::from(triangle.as_ref()));

				let node_index = flattened.len() as u32;
				flattened.push(BVHNodeGPU {
					bbox: AABBGPU::from(triangle.bounding_box()),
					left: u32::MAX,
					right: u32::MAX,
					triangle_id: triangle_index
				});

				node_index
			},
			BVHNode::Branch { left, right, bbox } => {
				let left_index = BVH::flatten(left, flattened, triangles);

				let right_index = if let Some(right_node) = right {
					BVH::flatten(right_node, flattened, triangles)
				} else {
					u32::MAX
				};

				let node_index = flattened.len() as u32;
				flattened.push(BVHNodeGPU {
					bbox: AABBGPU::from(*bbox),
					left: left_index,
					right: right_index,
					triangle_id: u32::MAX
				});

				node_index
			}
		}
	}
}

impl Hittable for BVH {
	fn hit(&self, ray: Ray, ray_t: Interval) -> Option<HitRecord> {
		self.root.hit(ray, ray_t)
	}

	fn bounding_box(&self) -> AABB {
		self.root.bounding_box()
	}
}

impl Hittable for BVHNode {
	fn hit(&self, ray: Ray, ray_t: Interval) -> Option<HitRecord> {
		match self {
			BVHNode::Leaf(object) => { object.hit(ray, ray_t) }
			BVHNode::Branch { left, right, bbox } => {
				if !bbox.hit(ray, ray_t) { return None; }

				if let Some(hit_left) = left.hit(ray, ray_t) {
					// left hit, check for closer right
					return if let Some(right) = &right {
						let hit_right = right.hit(
							ray,
							Interval::new(ray_t.min, hit_left.t)
						);
						hit_right.or(Some(hit_left))
					} else {
						Some(hit_left)
					}
				}

				if let Some(right) = &right {
					return right.hit(ray, ray_t);
				}

				None
			}
		}
	}

	fn bounding_box(&self) -> AABB {
		match self {
			BVHNode::Leaf(object) => { object.bounding_box() }
			BVHNode::Branch { left, right, bbox} => { *bbox }
		}
	}
}