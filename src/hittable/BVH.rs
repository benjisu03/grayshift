use crate::hittable::hittable::{HitRecord, Hittable, HittableList};
use crate::ray::Ray;
use crate::util::interval::Interval;
use crate::AABB::AABB;
use std::error::Error;

pub struct BVH {
	root: Box<BVHNode>,
}

enum BVHNode {
	Leaf(Box<dyn Hittable>),
	Branch {
		left: Box<BVHNode>,
		right: Option<Box<BVHNode>>,
		bbox: AABB
	}
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
 pub struct BVHNodeGPU {
	left: u32,
	right: u32,
	bbox: AABB
}

impl BVH {
	pub fn new(list: HittableList) -> Result<Self, Box<dyn Error>> {
		let mut objects = list.take_objects();
		if objects.is_empty() {
			return Err(Box::from("BVH cannot be built from empty list"));
		}

		let root = BVH::build(objects);
		Ok(BVH { root })
	}

	fn build(mut objects: Vec<Box<dyn Hittable>>) -> Box<BVHNode> {

		if objects.len() == 1 {
			let obj = objects.swap_remove(0);
			return Box::new(BVHNode::Leaf(obj));
		}

		let mut bbox = AABB::EMPTY;
		for object in objects.iter() {
			bbox = AABB::from_AABB_pair(bbox, object.bounding_box());
		}

		let axis = bbox.longest_axis();

		objects.sort_by(|a, b| {
			let a_min = a.bounding_box()[axis].min;
			let b_min = b.bounding_box()[axis].min;
			a_min.partial_cmp(&b_min).unwrap()
		});

		let middle = objects.len() / 2;
		let right_objs = objects.split_off(middle);
		let left_objs = objects;

		let left = BVH::build(left_objs);
		let right = Some(BVH::build(right_objs));

		Box::new(BVHNode::Branch { left, right, bbox })
	}

	pub fn to_gpu(&self) -> Vec<BVHNodeGPU> {
		let mut flattened = Vec::new();
		BVH::flatten(self.root.as_ref(), &mut flattened);
		flattened
	}

	fn flatten(node: &BVHNode, flattened: &mut Vec<BVHNodeGPU>) -> u32 {
		match node {
			BVHNode::Leaf(hittable) => {
				let node_index = flattened.len() as u32;
				flattened.push(BVHNodeGPU {
					left: u32::MAX,
					right: u32::MAX,
					bbox: hittable.bounding_box()
				});

				node_index
			},
			BVHNode::Branch { left, right, bbox } => {
				let left_index = BVH::flatten(left, flattened);

				let right_index = if let Some(right_node) = right {
					BVH::flatten(right_node, flattened)
				} else {
					u32::MAX
				};

				let node_index = flattened.len() as u32;
				flattened.push(BVHNodeGPU {
					left: left_index,
					right: right_index,
					bbox: *bbox
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