use crate::{
    ray::Ray,
    vec3::{Point3, Vec3},
};

#[derive(Debug)]
pub(crate) struct Camera {
    pub(crate) image_width: usize,
    pub(crate) image_height: usize,

    pub(crate) viewport_width: f64,
    pub(crate) viewport_height: f64,
    pub(crate) focal_length: f64,
    pub(crate) origin: Point3,
    pub(crate) horizontal: Vec3,
    pub(crate) vertical: Vec3,
    pub(crate) lower_left_corner: Vec3,
}

impl Camera {
    pub(crate) fn new(
        aspect_ratio: f64,
        image_width: usize,
        viewport_height: f64,
        focal_length: f64,
        origin: Point3,
    ) -> Self {
        let viewport_width = viewport_height * aspect_ratio;
        let horizontal = Vec3::from([viewport_width, 0.0, 0.0]);
        let vertical = Vec3::from([0.0, viewport_height, 0.0]);
        Camera {
            image_width,
            image_height: (image_width as f64 / aspect_ratio).ceil() as _,
            viewport_width,
            viewport_height,
            focal_length,
            origin,
            horizontal,
            vertical,
            lower_left_corner: origin
                - horizontal / 2.0
                - vertical / 2.0
                - Vec3::from([0.0, 0.0, focal_length]),
        }
    }

    pub(crate) fn get_ray(&self, u: f64, v: f64) -> Ray {
        Ray {
            orig: self.origin,
            dir: self.lower_left_corner + u * self.horizontal + v * self.vertical,
        }
    }
}
