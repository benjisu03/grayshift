use std::ops::Add;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Interval {
	pub min: f64,
	pub max: f64
}

impl Interval {
	pub const EMPTY: Interval = Interval { min: f64::MAX, max: f64::MIN };
	pub const UNIVERSE: Interval = Interval { min: f64::MIN, max: f64::MAX };
	pub const UNIT: Interval = Interval { min: 0.0, max: 1.0 };

	pub fn new(min: f64, max: f64) -> Self {
		Interval { min, max }
	}

	pub fn from_interval_pair(a: Interval, b: Interval) -> Self {
		Interval {
			min: if a.min <= b.min { a.min } else { b.min },
			max: if a.max >= b.max { a.max } else { b.max }
		}
	}

	pub fn size(&self) -> f64 {
		self.max - self.min
	}

	pub fn contains(&self, t: f64) -> bool {
		self.min <= t && t <= self.max
	}

	pub fn surrounds(&self, t: f64) -> bool {
		self.min < t && t < self.max
	}

	pub fn clamp(&self, t: f64) -> f64 {
		t.clamp(self.min, self.max)
	}

	pub fn expand(&self, delta: f64) -> Interval {
		let padding = delta / 2.0;
		Interval::new(self.min - padding, self.max + padding)
	}
}

impl Add<f64> for Interval {
	type Output = Self;

	fn add(self, rhs: f64) -> Self::Output {
		Interval {
			min: self.min + rhs,
			max: self.max + rhs
		}
	}
}