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
