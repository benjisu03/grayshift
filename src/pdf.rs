use std::f64::consts::PI;
use nalgebra::Vector3;
use crate::util::vec3::Vec3;

pub struct PDFSample<T> {
    pub sample: T,
    pub pdf: f32
}

pub trait PDF<T> {
    fn sample(&self) -> PDFSample<T>;
}

pub struct CosineWeightedPDF {}

impl PDF<Vector3<f32>> for CosineWeightedPDF {
    fn sample(&self) -> PDFSample<Vector3<f32>> {
        let r1 = fastrand::f32();
        let r2 = fastrand::f32();

        let phi = 2.0 * std::f32::consts::PI * r1;

        let (sin_phi, cos_phi) = phi.sin_cos();
        let cos_theta = (1.0 - r2).sqrt();
        let r2_sqrt = r2.sqrt();

        let x = cos_phi * r2_sqrt;
        let y = sin_phi * r2_sqrt;
        let z = cos_theta;

        let sample = Vector3::new(x, y, z);
        let pdf = cos_theta / std::f32::consts::PI;

        PDFSample { sample, pdf }
    }
}
