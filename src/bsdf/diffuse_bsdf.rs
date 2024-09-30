use std::f64::consts::PI;
use crate::material::Material;
use crate::util::vec3::Vec3;

pub struct Diffuse_BSDF {

}

impl Material for Diffuse_BSDF {

}

fn random_lambertian_direction() -> Vec3 {
    let r1 = fastrand::f64();
    let r2 = fastrand::f64();

    let phi = 2.0 * PI * r1;
    let r2_sqrt = r2.sqrt();
    let (sin_phi, cos_phi) = phi.sin_cos();

    let x = cos_phi * r2_sqrt;
    let y = sin_phi * r2_sqrt;
    let z = (1.0 - r2).sqrt();

    Vec3::new(x, y, z)
}

