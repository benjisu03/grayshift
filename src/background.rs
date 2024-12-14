use std::f64::consts::PI;
use crate::texture::Texture;
use crate::util::util::rotate_vector;
use crate::util::vec3::Vec3;

pub enum Background {
    SOLID(Vec3),
    HDRI(HDRI),
    CUBEMAP(CubeMap)
}

pub struct HDRI {
    pub image: radiant::Image,
    pub rotation: Vec3
}

impl HDRI {
    pub fn sample(&self, direction: Vec3) -> Vec3 {
        let rotated = rotate_vector(direction, self.rotation).unit();
        let theta = rotated.y.atan2(rotated.x);
        let phi = rotated.z.asin();

        let u = 0.5 + theta / (2.0 * PI);
        let v = 0.5 - phi / PI;

        let x = ((u * (self.image.width as f64)) as usize) % self.image.width;
        let y = ((v * (self.image.height as f64)) as usize) % self.image.height;

        let color = self.image.pixel(x, y);
        Vec3::new(color.r as f64, color.g as f64, color.b as f64)
    }
}

pub struct CubeMap {
    // L, R, U, D, F, B
    textures: [Box<dyn Texture>; 6]
}

impl CubeMap {
    pub fn new(textures: [Box<dyn Texture>; 6]) -> CubeMap {
        CubeMap { textures }
    }
    pub fn sample(&self, direction: Vec3) -> Vec3 {
        let (face, uv) = Self::get_face_uv(direction);

        let texture = &self.textures[face];
        texture.value_at(uv.0, uv.1, Vec3::new(1.0, 1.0, 1.0))
    }

    fn get_face_uv(direction: Vec3) -> (usize, (f64, f64)) {
        let abs = direction.abs();

        let (face, sc, tc, ma) = if abs.x >= abs.y && abs.x >= abs.z {
            if direction.x > 0.0 {
                (0, -direction.z, -direction.y, abs.x)
            } else {
                (1, direction.z, -direction.y, abs.x)
            }
        } else if abs.y >= abs.x && abs.y >= abs.z {
            if direction.y > 0.0 {
                (2, direction.x, direction.z, abs.y)
            } else {
                (3, direction.x, -direction.z, abs.y)
            }
        } else {
            if direction.z > 0.0 {
                (4, direction.x, -direction.y, abs.z)
            } else {
                (5, -direction.x, -direction.y, abs.z)
            }
        };

        let u = 0.5 * (sc / ma + 1.0);
        let v = 0.5 * (tc / ma + 1.0);

        (face, (u, v))
    }
}