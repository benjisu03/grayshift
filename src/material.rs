use std::f64::consts::PI;
use std::rc::Rc;
use crate::hittable::hittable::HitRecord;
use crate::ONB::OrthonormalBasis;
use crate::ray::Ray;
use crate::texture::{SolidColorTexture, Texture};
use crate::util::util::{random_cosine_direction, random_unit_vector};
use crate::util::vec3::Vec3;

pub trait Material {
	fn scatter(
		&self,
		ray_in: Ray,
		hit_record: &HitRecord
	) -> Option<ScatterRecord> { None }

	fn emitted(&self, u: f64, v: f64, p: Vec3) -> Vec3 { Vec3::ZERO }
	
	fn scattering_pdf(&self, ray_in: Ray, hit_record: &HitRecord, scattered: Ray) -> f64 { 0.0 }
}

pub struct ScatterRecord {
	pub scattered_ray: Ray,
	pub attenuation: Vec3,
	pub pdf: f64
}

pub struct Lambertian {
	texture: Rc<dyn Texture>
}


impl Lambertian {
	pub fn from_color(albedo: Vec3) -> Self {
		Lambertian { texture: Rc::new(SolidColorTexture::new(albedo)) }
	}

	pub fn from_texture(texture: Rc<dyn Texture>) -> Self {
		Lambertian { texture: texture.clone() }
	}
}

impl Material for Lambertian {
	fn scatter(&self, ray_in: Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
		let attenuation = self.texture.value_at(
			hit_record.u,
			hit_record.v,
			hit_record.position
		);

		let basis = OrthonormalBasis::new(hit_record.normal);
		let scatter_direction = basis.transform(random_cosine_direction()).unit();

		let scattered_ray = Ray::new(
			hit_record.position,
			scatter_direction,
			ray_in.time
		);

		let pdf = basis.w.dot(scatter_direction) / PI;

		Some(ScatterRecord {
			attenuation,
			scattered_ray,
			pdf
		})
	}

	fn scattering_pdf(&self, ray_in: Ray, hit_record: &HitRecord, scattered: Ray) -> f64 {
		1.0 / (2.0 * PI)
	}
}

pub struct Metal {
	albedo: Vec3,
	fuzz: f64
}

impl Metal {
	pub fn new(albedo: Vec3, fuzz: f64) -> Self {
		Metal { albedo, fuzz }
	}
}

impl Material for Metal {
	fn scatter(&self, ray_in: Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
		let mut reflected = ray_in.direction.reflect(hit_record.normal);
		reflected = reflected.unit() + self.fuzz * random_unit_vector();

		let scattered_ray = Ray::new(hit_record.position, reflected, ray_in.time);
		if scattered_ray.direction.dot(hit_record.normal) > 0.0 {
			return Some(ScatterRecord {
				attenuation: self.albedo,
				scattered_ray,
				pdf: 0.0
			});
		}

		None

	}
}

pub struct Dielectric {
	refraction_index: f64
}

impl Dielectric {
	pub fn new(refraction_index: f64) -> Self {
		Dielectric { refraction_index }
	}

	pub fn reflectance(cosine: f64, refraction_index: f64) -> f64 {
		// Schlick's approximation
		let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
		r0 = r0 * r0;
		r0 + (1.0 - r0) * f64::powi(1.0 - cosine, 5)
	}
}

impl Material for Dielectric {
	fn scatter(&self, ray_in: Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
		let ri = if hit_record.is_front_face {
			1.0 / self.refraction_index
		} else {
			self.refraction_index
		};

		let unit_direction = ray_in.direction.unit();
		let cos_theta = f64::min((-unit_direction).dot(hit_record.normal), 1.0);
		let sin_theta = f64::sqrt(1.0 - cos_theta * cos_theta);

		let cannot_refract = ri * sin_theta > 1.0;
		let fresnel_reflection = Self::reflectance(cos_theta, ri) > fastrand::f64();
		let direction = if cannot_refract || fresnel_reflection {
			unit_direction.reflect(hit_record.normal)
		} else {
			unit_direction.refract(hit_record.normal, ri)
		};


		Some(ScatterRecord {
			attenuation: Vec3::new(1.0, 1.0, 1.0),
			scattered_ray: Ray::new(hit_record.position, direction, ray_in.time),
			pdf: 0.0
		})
	}
}

pub struct DiffuseLight {
	texture: Rc<dyn Texture>
}

impl DiffuseLight {
	pub fn new(texture: Rc<dyn Texture>) -> Self {
		DiffuseLight { texture }
	}

	pub fn from_color(color: Vec3) -> Self {
		DiffuseLight { texture: Rc::new(SolidColorTexture::new(color) )}
	}
}

impl Material for DiffuseLight {
	fn emitted(&self, u: f64, v: f64, p: Vec3) -> Vec3 {
		self.texture.value_at(u, v, p)
	}
}

pub struct Isotropic {
	texture: Rc<dyn Texture>
}

impl Isotropic {
	pub fn new(texture: Rc<dyn Texture>) -> Self {
		Isotropic { texture }
	}

	pub fn from_color(color: Vec3) -> Self {
		Isotropic { texture: Rc::new(SolidColorTexture::new(color) )}
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

	fn scattering_pdf(&self, ray_in: Ray, hit_record: &HitRecord, scattered: Ray) -> f64 {
		1.0 / (4.0 * PI)
	}
}