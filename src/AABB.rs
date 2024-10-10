use std::ops::{Add, Index};
use nalgebra::Vector3;
use crate::ray::Ray;
use crate::util::interval::Interval;
use crate::util::vec3::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct AABB {
	pub x: Interval,
	pub y: Interval,
	pub z: Interval
}

impl AABB {
	pub const EMPTY: AABB = AABB {
		x: Interval::EMPTY,
		y: Interval::EMPTY,
		z: Interval::EMPTY
	};

	pub fn new(x: Interval, y: Interval, z: Interval) -> Self {
		AABB { x, y, z }
	}

	pub fn from_corners(a: Vector3<f32>, b: Vector3<f32>) -> Self {
		let x = if a.x <= b.x {
			Interval::new(a.x, b.x)
		} else {
			Interval::new(b.x, a.x)
		};

		let y = if a.y <= b.y {
			Interval::new(a.y, b.y)
		} else {
			Interval::new(b.y, a.y)
		};

		let z = if a.z <= b.z {
			Interval::new(a.z, b.z)
		} else {
			Interval::new(b.z, a.z)
		};

		let mut aabb = AABB { x, y, z };
		aabb.pad_to_minimums();
		
		aabb
	}

	pub fn from_AABB_pair(a: AABB, b: AABB) -> Self {
		AABB {
			x: Interval::from_interval_pair(a.x, b.x),
			y: Interval::from_interval_pair(a.y, b.y),
			z: Interval::from_interval_pair(a.z, b.z),

		}
	}

	pub fn hit(&self, ray: Ray, ray_t: Interval) -> bool {
		let mut interval = ray_t;

		// x
		{
			let ad_inv = 1.0 / ray.direction.x;
			let t0 = (self.x.min - ray.origin.x) * ad_inv;
			let t1 = (self.x.max - ray.origin.x) * ad_inv;

			if t0 < t1 {
				if t0 > interval.min { interval.min = t0; }
				if t1 < interval.max { interval.max = t1; }
			} else {
				if t1 > interval.min { interval.min = t1; }
				if t0 < interval.max { interval.max = t0; }
			}

			if interval.max <= interval.min { return false; }
		}

		// y
		{
			let ad_inv = 1.0 / ray.direction.y;
			let t0 = (self.y.min - ray.origin.y) * ad_inv;
			let t1 = (self.y.max - ray.origin.y) * ad_inv;

			if t0 < t1 {
				if t0 > interval.min { interval.min = t0; }
				if t1 < interval.max { interval.max = t1; }
			} else {
				if t1 > interval.min { interval.min = t1; }
				if t0 < interval.max { interval.max = t0; }
			}

			if interval.max <= interval.min { return false; }
		}

		// z
		{
			let ad_inv = 1.0 / ray.direction.z;
			let t0 = (self.z.min - ray.origin.z) * ad_inv;
			let t1 = (self.z.max - ray.origin.z) * ad_inv;

			if t0 < t1 {
				if t0 > interval.min { interval.min = t0; }
				if t1 < interval.max { interval.max = t1; }
			} else {
				if t1 > interval.min { interval.min = t1; }
				if t0 < interval.max { interval.max = t0; }
			}

			if interval.max <= interval.min { return false; }
		}

		true
	}

	pub fn longest_axis(&self) -> usize {
		if self.x.size() > self.y.size() {
			if self.x.size() > self.z.size() { 0 } else { 2 }
		} else {
			if self.y.size() > self.z.size() { 1 } else { 2 }
		}
	}

	fn pad_to_minimums(&mut self) {
		let delta = 0.0001;
		if self.x.size() < delta { self.x = self.x.expand(delta) }
		if self.y.size() < delta { self.y = self.y.expand(delta) }
		if self.z.size() < delta { self.z = self.z.expand(delta) }
	}
}

impl Index<usize> for AABB {
	type Output = Interval;

	fn index(&self, index: usize) -> &Self::Output {
		match index {
			0 => &self.x,
			1 => &self.y,
			2 => &self.z,
			_ => panic!("index out of bounds")
		}
	}
}

impl Add<Vector3<f32>> for AABB {
	type Output = Self;

	fn add(self, rhs: Vector3<f32>) -> Self::Output {
		AABB {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
			z: self.z + rhs.z
		}
	}
}