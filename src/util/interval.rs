use std::ops::Add;

#[derive(Copy, Clone, Debug)]
pub struct Interval {
	pub min: f32,
	pub max: f32
}

impl Interval {
	pub const EMPTY: Interval = Interval { min: f32::MAX, max: f32::MIN };
	pub const UNIVERSE: Interval = Interval { min: f32::MIN, max: f32::MAX };
	pub const UNIT: Interval = Interval { min: 0.0, max: 1.0 };

	pub fn new(min: f32, max: f32) -> Self {
		Interval { min, max }
	}

	pub fn from_interval_pair(a: Interval, b: Interval) -> Self {
		Interval {
			min: if a.min <= b.min { a.min } else { b.min },
			max: if a.max >= b.max { a.max } else { b.max }
		}
	}

	pub fn size(&self) -> f32 {
		self.max - self.min
	}

	pub fn contains(&self, t: f32) -> bool {
		self.min <= t && t <= self.max
	}

	pub fn surrounds(&self, t: f32) -> bool {
		self.min < t && t < self.max
	}

	pub fn clamp(&self, t: f32) -> f32 {
		t.clamp(self.min, self.max)
	}

	pub fn expand(&self, delta: f32) -> Interval {
		let padding = delta / 2.0;
		Interval::new(self.min - padding, self.max + padding)
	}
}

impl Add<f32> for Interval {
	type Output = Self;

	fn add(self, rhs: f32) -> Self::Output {
		Interval {
			min: self.min + rhs,
			max: self.max + rhs
		}
	}
}