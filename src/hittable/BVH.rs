use std::rc::Rc;
use crate::AABB::AABB;
use crate::hittable::hittable::{HitRecord, Hittable, HittableList};
use crate::ray::Ray;
use crate::util::interval::Interval;

pub struct BVH {
	root: BVHNode,
}

enum BVHNode {
	Leaf(Box<dyn Hittable>),
	Branch {
		left: BVHNode::Leaf(_),
		right: Option<BVHNode::Leaf(_)>,
		bbox: AABB
	}
}

#[repr(C)]
struct BVHNodeGPU {
	left: u32,
	right: u32,
	bbox: AABB
}

impl BVH {
	pub fn new(list: HittableList) -> Result<Self, Self::Error> {
		let mut objects = list.take_objects();
		if objects.is_empty() {
			return Err("BVH cannot be built from empty list");
		}

		let root = BVH::build(objects);
		Ok(BVH { root })
	}

	fn build(mut objects: Vec<Box<dyn Hittable>>) -> BVHNode {

		if objects.len() == 1 {
			let obj = objects.swap_remove(0);
			return BVHNode::Leaf(obj);
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
		let right = BVH::build(right_objs);

		BVHNode::Branch { left, right: Some(right), bbox }
	}

	pub fn to_gpu(&self) -> Vec<BVHNodeGPU> {
		let mut flattened = Vec::new();

	}

	fn flatten(node: BVHNode, flattened: &mut Vec<BVHNodeGPU>) -> u32 {
		let node_index = flattened.len() as u32;
		match node {

		}

		node_index
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