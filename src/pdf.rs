use std::f64::consts::PI;
use std::sync::Arc;
use crate::hittable::hittable::Hittable;
use crate::ONB::OrthonormalBasis;
use crate::util::vec3::Vec3;

pub struct PDFSample<T> {
    pub sample: T,
    pub pdf: f64
}

pub trait PDF<T> {
    fn sample(&self) -> PDFSample<T>;
}

pub struct CosineWeightedPDF {
    basis: OrthonormalBasis
}

impl CosineWeightedPDF {
    pub fn new(normal: Vec3) -> Self {
        CosineWeightedPDF { basis: OrthonormalBasis::new(normal) }
    }
}

impl PDF<Vec3> for CosineWeightedPDF {
    fn sample(&self) -> PDFSample<Vec3> {
        let r1 = fastrand::f64();
        let r2 = fastrand::f64();

        let phi = 2.0 * PI * r1;

        let (sin_phi, cos_phi) = phi.sin_cos();
        let cos_theta = (1.0 - r2).sqrt();
        let r2_sqrt = r2.sqrt();

        let x = cos_phi * r2_sqrt;
        let y = sin_phi * r2_sqrt;
        let z = cos_theta;

        let direction = Vec3::new(x, y, z);

        let sample = self.basis.transform(direction);
        let pdf = cos_theta / PI;

        PDFSample { sample, pdf }
    }
}


pub struct HittablePDF {
    pub hittable: Arc<dyn Hittable>,
    pub origin: Vec3
}

impl HittablePDF {
    pub fn new(hittable: Arc<dyn Hittable>, origin: Vec3) -> Self {
        HittablePDF { hittable, origin }
    }
}

impl PDF<Vec3> for HittablePDF {
    fn sample(&self) -> PDFSample<Vec3> {
        self.hittable.sample_surface(self.origin)
    }
}

pub struct MixturePDF<T> {
    pub pdf1: Arc<dyn PDF<T>>,
    pub pdf2: Arc<dyn PDF<T>>
}

impl<T> MixturePDF<T> {
    pub fn new(pdf1: Arc<dyn PDF<T>>, pdf2: Arc<dyn PDF<T>>) -> Self {
        MixturePDF { pdf1, pdf2 }
    }
}

impl<T> PDF<T> for MixturePDF<T> {
    fn sample(&self) -> PDFSample<T> {
        let sample1 = self.pdf1.sample();
        let sample2 = self.pdf2.sample();

        let sample = if fastrand::f64() < 0.5 {
            sample1.sample
        } else {
            sample2.sample
        };

        let pdf = 0.5 * sample1.pdf + 0.5 * sample2.pdf;

        PDFSample { sample, pdf }
    }
}