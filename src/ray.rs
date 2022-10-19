use std::rc::Rc;

use crate::material::{self, Material};
use crate::vec3::{Point3, Vec3};

#[derive(Debug)]
pub(crate) struct Ray {
    pub(crate) orig: Point3,
    pub(crate) dir: Vec3,
}

impl Ray {
    pub(crate) fn at(&self, t: f64) -> Point3 {
        self.orig + t * self.dir
    }
}

#[derive(Debug)]
pub(crate) enum Face {
    Front,
    Back,
}

#[derive(Debug)]
pub(crate) struct HitRecord {
    pub(crate) p: Point3,
    pub(crate) normal: Vec3,
    pub(crate) t: f64,
    pub(crate) face: Face,
    pub(crate) mat: Rc<dyn Material>,
}

impl HitRecord {
    pub(crate) fn new(
        p: Point3,
        outward_normal: Vec3,
        t: f64,
        ray: &Ray,
        mat: Rc<dyn Material>,
    ) -> Self {
        let (normal, face) = if ray.dir.dot(&outward_normal) > 0.0 {
            (-outward_normal, Face::Back)
        } else {
            (outward_normal, Face::Front)
        };
        Self {
            p,
            normal,
            t,
            face,
            mat,
        }
    }
}

pub(crate) trait Hittable {
    fn hit(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<HitRecord>;
}
