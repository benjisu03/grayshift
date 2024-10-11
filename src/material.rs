use crate::hittable::hittable::HitRecord;
use crate::pdf::{CosineWeightedPDF, PDF};
use crate::ray::Ray;
use crate::texture::{SolidColorTexture, Texture};
use crate::ONB::OrthonormalBasis;
use nalgebra::Vector3;
use std::f32::consts::PI;
use std::sync::Arc;
use crate::util::util::{random_unit_vector, reflect, refract};

pub trait Material: Send + Sync {
	fn scatter(
		&self,
		ray_in: Ray,
		hit_record: &HitRecord
	) -> Option<ScatterRecord> { None }

	fn emitted(&self, u: f32, v: f32, p: Vector3<f32>) -> Vector3<f32> { Vector3::new(0.0, 0.0, 0.0) }
	
	fn scattering_pdf(&self, ray_in: &Ray, hit_record: &HitRecord, scattered: &Ray) -> f32 { 0.0 }
}

pub struct ScatterRecord {
	pub attenuation: Vector3<f32>,
	pub scattered_ray: Ray,
	pub pdf: f32
}

pub struct Lambertian {
	texture: Arc<dyn Texture>
}


impl Lambertian {
	pub fn from_color(albedo: Vector3<f32>) -> Self {
		Lambertian { texture: Arc::new(SolidColorTexture::new(albedo)) }
	}

	pub fn from_texture(texture: Arc<dyn Texture>) -> Self {
		Lambertian { texture: texture.clone() }
	}
}

impl Material for Lambertian {
	fn scatter(&self, ray_in: Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
		let mut attenuation = self.texture.value_at(
			hit_record.u,
			hit_record.v,
			hit_record.position
		);

		attenuation /= std::f32::consts::PI;

		let pdf_sample = (CosineWeightedPDF{}).sample();
		let pdf = pdf_sample.pdf;

		// Reorient scatter around normal
		let basis = OrthonormalBasis::new(hit_record.normal);
		let scatter_direction = basis.transform(pdf_sample.sample).normalize();

		let scattered_ray = Ray::new(
			hit_record.position,
			scatter_direction,
			ray_in.time
		);

		Some(ScatterRecord {
			attenuation,
			scattered_ray,
			pdf
		})
	}
}

pub struct Metal {
	albedo: Vector3<f32>,
	fuzz: f32
}

impl Metal {
	pub fn new(albedo: Vector3<f32>, fuzz: f32) -> Self {
		Metal { albedo, fuzz }
	}
}

impl Material for Metal {
	fn scatter(&self, ray_in: Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
		let mut reflected = reflect(&ray_in.direction, &hit_record.normal);
		reflected = reflected.normalize() + self.fuzz * random_unit_vector();

		let scattered_ray = Ray::new(hit_record.position, reflected, ray_in.time);
		if scattered_ray.direction.dot(&hit_record.normal) > 0.0 {
			return Some(ScatterRecord {
				attenuation: self.albedo,
				scattered_ray,
				pdf: 1.0
			});
		}

		None

	}
}

pub struct Dielectric {
	refraction_index: f32
}

impl Dielectric {
	pub fn new(refraction_index: f32) -> Self {
		Dielectric { refraction_index }
	}

	pub fn reflectance(cosine: f32, refraction_index: f32) -> f32 {
		// Schlick's approximation
		let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
		r0 = r0 * r0;
		r0 + (1.0 - r0) * f32::powi(1.0 - cosine, 5)
	}
}

impl Material for Dielectric {
	fn scatter(&self, ray_in: Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
		let ri = if hit_record.is_front_face {
			1.0 / self.refraction_index
		} else {
			self.refraction_index
		};

		let unit_direction = ray_in.direction.normalize();
		let cos_theta = f32::min((-unit_direction).dot(&hit_record.normal), 1.0);
		let sin_theta = f32::sqrt(1.0 - cos_theta * cos_theta);

		let cannot_refract = ri * sin_theta > 1.0;
		let fresnel_reflection = Self::reflectance(cos_theta, ri) > fastrand::f32();
		let direction = if cannot_refract || fresnel_reflection {
			reflect(&unit_direction, &hit_record.normal)
		} else {
			refract(&unit_direction, &hit_record.normal, ri)
		};


		Some(ScatterRecord {
			attenuation: Vector3::new(1.0, 1.0, 1.0),
			scattered_ray: Ray::new(hit_record.position, direction, ray_in.time),
			pdf: 1.0
		})
	}
}

pub struct DiffuseLight {
	texture: Arc<dyn Texture>
}

impl DiffuseLight {
	pub fn new(texture: Arc<dyn Texture>) -> Self {
		DiffuseLight { texture }
	}

	pub fn from_color(color: Vector3<f32>) -> Self {
		DiffuseLight { texture: Arc::new(SolidColorTexture::new(color) )}
	}
}

impl Material for DiffuseLight {
	fn emitted(&self, u: f32, v: f32, p: Vector3<f32>) -> Vector3<f32> {
		self.texture.value_at(u, v, p)
	}
}

pub struct Isotropic {
	texture: Arc<dyn Texture>
}

impl Isotropic {
	pub fn new(texture: Arc<dyn Texture>) -> Self {
		Isotropic { texture }
	}

	pub fn from_color(color: Vector3<f32>) -> Self {
		Isotropic { texture: Arc::new(SolidColorTexture::new(color) )}
	}
}

impl Material for Isotropic {
	fn scatter(&self, ray_in: Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
		Some(ScatterRecord {
			attenuation: self.texture.value_at(
				hit_record.u,
				hit_record.v,
				hit_record.position
			),
			scattered_ray: Ray::new(hit_record.position, random_unit_vector(), ray_in.time),
			pdf: 1.0 / (4.0 * PI)
		})
	}

	fn scattering_pdf(&self, ray_in: &Ray, hit_record: &HitRecord, scattered: &Ray) -> f32 {
		1.0 / (4.0 * PI)
	}
}

pub struct EmptyMaterial {}

impl EmptyMaterial {
	pub fn new() -> Self { EmptyMaterial {} }
}

impl Material for EmptyMaterial {}