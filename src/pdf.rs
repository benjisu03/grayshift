use std::f64::consts::PI;
use crate::ONB::OrthonormalBasis;
use crate::util::vec3::Vec3;

pub trait PDF<T> {
    fn sample(&self) -> PDFSample<T>;
}

pub struct PDFSample<T> {
    pub sample: T,
    pub weight: f64
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
        let r2_sqrt = r2.sqrt();
        let (cos_phi, sin_phi) = phi.sin_cos();

        let x = cos_phi * r2_sqrt;
        let y = sin_phi * r2_sqrt;
        let z = f64::sqrt(1.0 - r2);

        let v = Vec3::new(x, y, z);
        let sample = self.basis.transform(v);

        let cos_theta = sample.unit().dot(self.basis.w);
        let weight = (cos_theta / PI).max(0.0);

        PDFSample { sample, weight }
    }
}