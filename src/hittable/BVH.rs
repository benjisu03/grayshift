use std::rc::Rc;
use crate::AABB::AABB;
use crate::hittable::hittable::{HitRecord, Hittable, HittableList};
use crate::ray::Ray;
use crate::util::interval::Interval;

pub struct BVHNode {
	left: Box<dyn Hittable>,
	right: Option<Box<dyn Hittable>>,
	bbox: AABB
}

impl BVHNode {

	pub fn from_list(mut list: HittableList) -> Box<Self> {
		BVHNode::construct_tree(list.objects)
	}
	pub fn construct_tree(mut objects: Vec<Box<dyn Hittable>>) -> Box<BVHNode> {

		if objects.len() == 1 {
			let obj = objects.swap_remove(0);

			return Box::new(BVHNode {
				bbox: obj.bounding_box(),
				left: obj,
				right: None,
			});
		}

		if objects.len() == 2 {
			let obj_right = objects.swap_remove(1);
			let obj_left = objects.swap_remove(0);

			return Box::new(BVHNode {
				bbox: AABB::from_AABB_pair(
					obj_left.bounding_box(),
					obj_right.bounding_box()
				),
				left: obj_left,
				right: Some(obj_right),
			});
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

		let left = Self::construct_tree(left_objs);
		let right = Self::construct_tree(right_objs);

		Box::new(BVHNode { left, right: Some(right), bbox })
	}
}

impl Hittable for BVHNode {
	fn hit(&self, ray: Ray, ray_t: Interval) -> Option<HitRecord> {
		if !self.bbox.hit(ray, ray_t) { return None; }

		if let Some(hit_left) = self.left.hit(ray, ray_t) {
			// left hit, check for closer right
			return if let Some(right) = &self.right {
				let hit_right = right.hit(
					ray,
					Interval::new(ray_t.min, hit_left.t)
				);
				hit_right.or(Some(hit_left))
			} else {
				Some(hit_left)
			}
		}

		if let Some(right) = &self.right {
			return right.hit(ray, ray_t);
		}

		None
	}

	fn bounding_box(&self) -> AABB {
		self.bbox
	}
}