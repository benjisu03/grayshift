use std::f64::consts::PI;
use crate::ONB::OrthonormalBasis;
use crate::util::util::{random_cosine_direction, random_unit_vector};
use crate::util::vec3::Vec3;

pub trait PDF {
    fn value(&self, direction: Vec3) -> f64;
    fn generate(&self) -> Vec3;
}

pub struct SpherePDF {}

impl SpherePDF {}

impl PDF for SpherePDF {
    fn value(&self, direction: Vec3) -> f64 {
        1.0 / (4.0 * PI)
    }

    fn generate(&self) -> Vec3 {
        random_unit_vector()
    }
}

pub struct CosinePDF {
    basis: OrthonormalBasis
}

impl CosinePDF {
    pub fn new(normal: Vec3) -> CosinePDF {
        CosinePDF { basis: OrthonormalBasis::new(normal) }
    }
}

impl PDF for CosinePDF {
    fn value(&self, direction: Vec3) -> f64 {
        let cos_theta = direction.unit().dot(self.basis.w);
        (cos_theta / PI).max(0.0)
    }

    fn generate(&self) -> Vec3 {
        self.basis.transform(random_cosine_direction())
    }
}
