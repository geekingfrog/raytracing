use std::rc::Rc;

use crate::{
    ray::{HitRecord, Hittable, Ray},
    vec3::{Color, Point3, Vec3},
};

pub(crate) trait Material {
    /// produce a scattered ray (if not completely absorbed)
    /// and say by how much it should be attenuated
    fn scatter(&self, ray_in: &Ray, hit: &HitRecord) -> Option<(Ray, Color)>;

    /// for debugging purposes
    fn name(&self) -> &'static str;
}

impl std::fmt::Debug for dyn Material {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Debug)]
pub(crate) struct Sphere {
    pub(crate) center: Point3,
    pub(crate) radius: f64,
    pub(crate) material: Rc<dyn Material>,
}

// impl std::fmt::Debug for Sphere {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Sphere")
//             .field("center", &self.center)
//             .field("radius", &self.radius)
//             .field("material", &self.material.name())
//             .finish()
//     }
// }

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<HitRecord> {
        let oc = ray.orig - self.center;
        let a = ray.dir.length_squared();
        let half_b = oc.dot(&ray.dir);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();
        // find the nearest root that lies in the acceptable range
        let mut root = (-half_b - sqrtd) / a;
        if root < tmin || tmax < root {
            root = (-half_b + sqrtd) / a;
            if root < tmin || tmax < root {
                return None;
            }
        }

        let p = ray.at(root);
        let outward_normal = (p - self.center) / self.radius;
        Some(HitRecord::new(
            p,
            outward_normal,
            root,
            ray,
            Rc::clone(&self.material),
        ))
    }
}

impl<T> Hittable for Vec<T>
where
    T: Hittable,
{
    fn hit(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<HitRecord> {
        let mut closest_so_far = tmax;
        let mut hit = None;

        for obj in self {
            if let Some(obj_hit) = obj.hit(ray, tmin, closest_so_far) {
                closest_so_far = obj_hit.t;
                hit = Some(obj_hit);
            }
        }
        hit
    }
}

pub(crate) struct Lambertian {
    pub(crate) albedo: Color,
}

impl Material for Lambertian {
    // lambertian reflection, which has a distribution of cos(Φ)
    // this leads to less pronounced shadows, and lighter spheres.
    fn scatter(&self, _ray_in: &Ray, hit: &HitRecord) -> Option<(Ray, Color)> {
        let mut scatter_direction = hit.normal + Vec3::random_unit_vector();
        if scatter_direction.is_near_zero() {
            scatter_direction = hit.normal;
        }
        let scattered = Ray {
            orig: hit.p,
            dir: scatter_direction,
        };
        let attenuation = self.albedo;
        Some((scattered, attenuation))
    }

    fn name(&self) -> &'static str {
        "Lambertian"
    }
}

pub(crate) struct Metal {
    pub(crate) albedo: Color,
}

impl Material for Metal {
    fn scatter(&self, ray_in: &Ray, hit: &HitRecord) -> Option<(Ray, Color)> {
        let v = ray_in.dir.unit();
        let reflected = v - 2.0 * v.dot(&hit.normal) * hit.normal;
        if reflected.dot(&hit.normal) > 0.0 {
            let scattered = Ray {
                orig: hit.p,
                dir: reflected,
            };
            let attenuation = self.albedo;
            Some((scattered, attenuation))
        } else {
            None
        }
    }

    fn name(&self) -> &'static str {
        "Metal"
    }
}
