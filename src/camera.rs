use std::f64::consts::PI;

use crate::{
    ray::Ray,
    vec3::{Point3, Vec3},
};

#[derive(Debug, Clone, PartialEq)]
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

    u: Vec3,
    v: Vec3,
    lens_radius: f64,
}

impl Camera {
    pub(crate) fn new(
        look_from: Point3,
        look_at: Point3,
        vup: Vec3, // where does is up for the camera (rotation around the direction of look_at)
        vfof: f64, // vertical field-of-view in degrees
        aspect_ratio: f64,
        image_width: usize,
        focal_length: f64,
        aperture: f64,
        focus_dist: f64,
    ) -> Self {
        let theta = vfof * PI / 180.0;
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = viewport_height * aspect_ratio;

        let w = (look_from - look_at).unit();
        let u = (vup.cross(&w)).unit();
        let v = w.cross(&u);

        let origin = look_from;
        let horizontal = focus_dist * viewport_width * u; // Vec3::from([viewport_width, 0.0, 0.0]);
        let vertical = focus_dist * viewport_height * v; // Vec3::from([0.0, viewport_height, 0.0]);
        let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - focus_dist * w;
        let lens_radius = aperture / 2.0;
        Camera {
            image_width,
            image_height: (image_width as f64 / aspect_ratio).ceil() as _,
            viewport_width,
            viewport_height,
            focal_length,
            origin,
            horizontal,
            vertical,
            lower_left_corner,
            u,
            v,
            lens_radius,
        }
    }

    pub(crate) fn get_ray(&self, s: f64, t: f64) -> Ray {
        let rd = self.lens_radius * Vec3::random_in_unit_disk();
        let offset = self.u * rd.x + self.v * rd.y;
        Ray {
            orig: self.origin + offset,
            dir: self.lower_left_corner + s * self.horizontal + t * self.vertical
                - self.origin
                - offset,
        }
    }
}
