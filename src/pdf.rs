use std::f64::consts::PI;
use crate::util::vec3::Vec3;

pub struct PDFSample<T> {
    pub sample: T,
    pub pdf: f64
}

pub trait PDF<T> {
    fn sample(&self) -> PDFSample<T>;
}

pub struct CosineWeightedPDF {}

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

        let sample = Vec3::new(x, y, z);
        let pdf = cos_theta / PI;

        PDFSample { sample, pdf }
    }
}
