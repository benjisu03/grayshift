use std::f64::consts::PI;
use std::sync::Arc;
use crate::hittable::hittable::Hittable;
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

pub struct HittablePDF {
    hittable: Arc<dyn Hittable>,
    origin: Vec3
}

impl HittablePDF {
    pub fn new(hittable: Arc<dyn Hittable>, origin: Vec3) -> HittablePDF {
        HittablePDF { hittable, origin }
    }
}

impl PDF for HittablePDF {
    fn value(&self, direction: Vec3) -> f64 {
        self.hittable.pdf_value(self.origin, direction)
    }

    fn generate(&self) -> Vec3 {
        self.hittable.random(self.origin)
    }
}