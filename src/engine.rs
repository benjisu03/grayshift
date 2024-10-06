use std::error::Error;
use std::sync::Arc;
use indicatif::ProgressBar;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use crate::background::Background;
use crate::camera::Camera;
use crate::color::luminance;
use crate::hittable::hittable::Hittable;
use crate::output::RenderTarget;
use crate::ray::Ray;
use crate::util::interval::Interval;
use crate::util::vec3::Vec3;

pub struct Engine {
    camera: Camera,
    render_target: Box<dyn RenderTarget>,
    background: Box<dyn Background>,
    render_settings: RenderSettings,

    batch_sqrt: u32,
    batch_sqrt_recip: f64
}

impl Engine {
    pub fn new(
        camera: Camera,
        render_target: Box<dyn RenderTarget>,
        background: Box<dyn Background>,
        render_settings: RenderSettings
    ) -> Engine {

        let batch_sqrt = (render_settings.sample_settings.batch_size as f64).sqrt() as u32;
        let batch_sqrt_recip = 1.0 / (batch_sqrt as f64);

        Engine {
            camera,
            render_target,
            background,
            render_settings,

            batch_sqrt,
            batch_sqrt_recip
        }
    }

    pub fn render(&mut self, world: Box<dyn Hittable>, lights: Arc<dyn Hittable>) -> Result<(), Box<dyn Error>> {
        self.render_target.init()?;

        let (image_width, image_height) = self.render_target.size();

        let pixels:Vec<u32> = (0..(image_width * image_height)).collect();
        let mut colors = Vec::with_capacity(pixels.len());

        let progress = Arc::new(ProgressBar::new(pixels.len() as u64));

        pixels.par_iter().map(|n| {
            let i = n % image_width;
            let j = n / image_width;
            self.sample(i, j, &world, lights.clone(), progress.clone())
        }).collect_into_vec(&mut colors);

        for color in colors {
            self.render_target.write_color(color)?;
        }

        Ok(())
    }

    fn sample(&self, i: u32, j: u32, world: &Box<dyn Hittable>, lights: Arc<dyn Hittable>, progress: Arc<ProgressBar>) -> Vec3 {
        let mut pixel_color = Vec3::ZERO;

        let tolerance_sq = self.render_settings.sample_settings.tolerance * self.render_settings.sample_settings.tolerance;
        let confidence_sq = self.render_settings.sample_settings.confidence * self.render_settings.sample_settings.confidence;

        let mut sum = 0.0;
        let mut sq_sum = 0.0;
        let mut sample_count = 0.0;

        loop {
            // do a batch of samples
            sample_count += self.render_settings.sample_settings.batch_size as f64;

            for s_i in 0..self.batch_sqrt {
                for s_j in 0..self.batch_sqrt {
                    let offset = self.stratified_square_sample(s_i, s_j);
                    let ray = self.camera.get_ray(offset, i, j);

                    let sample_color = self.ray_color(ray, self.render_settings.max_ray_depth, &world, lights.clone());
                    pixel_color += sample_color;

                    // luminance allows 1D tolerance based on human perception
                    let lum = luminance(sample_color);
                    sum += lum;
                    sq_sum += lum * lum;
                }
            }

            // calculate variance
            let mean = sum / sample_count;
            let variance_sq = 1.0 / (sample_count - 1.0) * (sq_sum - sum * sum / sample_count);

            let convergence_sq = confidence_sq * variance_sq / sample_count;

            // check if convergence is within tolerance, squared to reduce calculations
            if convergence_sq < (mean * mean * tolerance_sq) {
                break;
            }

            // some pixels take too long to converge
            // this means tolerance is not guaranteed, but adaptive sampling will at least speed up easy pixels
            if sample_count as u32 > self.render_settings.sample_settings.max_samples {
                break;
            }
        }

        pixel_color /= sample_count;

        progress.inc(1);
        pixel_color
    }

    // PRIVATE //

    fn stratified_square_sample(&self, s_i: u32, s_j: u32) -> Vec3 {
        Vec3::new(
            ((s_i as f64) + fastrand::f64()) * self.batch_sqrt_recip - 0.5,
            ((s_j as f64) + fastrand::f64()) * self.batch_sqrt_recip - 0.5,
            0.0
        )
    }

    fn ray_color(&self, ray: Ray, depth: u32, world: &Box<dyn Hittable>, lights: Arc<dyn Hittable>) -> Vec3 {
        if depth <= 0 { return Vec3::ZERO }

        if let Some(hit_record) = world.hit(ray, Interval::new(0.001, f64::MAX)) {
            let emission_color = hit_record.material.emitted(
                hit_record.u,
                hit_record.v,
                hit_record.position
            );

            let material = hit_record.material.as_ref();
            if let Some(scatter_record) = material.scatter(ray, &hit_record) {

                let scatter_color = self.ray_color(
                    scatter_record.scattered_ray,
                    depth - 1,
                    world,
                    lights.clone()
                );

                let cos_theta = hit_record.normal.dot(scatter_record.scattered_ray.direction);

                let color_from_scatter = scatter_color * scatter_record.attenuation * cos_theta / scatter_record.pdf;

                return emission_color + color_from_scatter;
            }

            return emission_color;
        }

        self.background.sample(ray.direction)
    }
}

pub struct RenderSettings {
    pub sample_settings: SampleSettings,
    pub max_ray_depth: u32
}

pub struct SampleSettings {
    pub confidence: f64, // z-value
    pub tolerance: f64,
    pub batch_size: u32,
    pub max_samples: u32
}