use std::rc::Rc;

use crate::{
    ray::{Face, HitRecord, Hittable, Ray},
    vec3::{Color, Point3, Vec3},
};

// pub(crate) trait Material {
//     /// produce a scattered ray (if not completely absorbed)
//     /// and say by how much it should be attenuated
//     fn scatter(&self, ray_in: &Ray, hit: &HitRecord) -> Option<(Ray, Color)>;
//
//     /// for debugging purposes
//     fn name(&self) -> &'static str;
// }
//
// impl std::fmt::Debug for dyn Material {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.write_str(self.name())
//     }
// }

#[derive(Debug, Clone)]
pub(crate) enum Material {
    Lambertian {
        albedo: Color,
    },
    /// fuzz should be in [0;1]
    Metal {
        albedo: Color,
        fuzz: f64,
    },
    Dielectric {
        /// index of refraction
        ir: f64,
    },
}

fn reflect(v: &Vec3, n: &Vec3) -> Vec3 {
    v - 2.0 * v.dot(n) * n
}

fn refract(uv: &Vec3, n: &Vec3, etai_over_etat: f64) -> Vec3 {
    let cos_theta = (-uv).dot(n).min(1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -(1.0 - r_out_perp.length_squared()).abs().sqrt() * n;
    r_out_perp + r_out_parallel
}

fn reflectance(cos: f64, ref_idx: f64) -> f64 {
    //Schlick's approximation for reflectance
    let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cos).powf(5.0)
}

impl Material {
    pub(crate) fn scatter(&self, ray_in: &Ray, hit: &HitRecord) -> Option<(Ray, Color)> {
        match self {
            Material::Lambertian { albedo } => {
                let mut scatter_direction = hit.normal + Vec3::random_unit_vector();
                if scatter_direction.is_near_zero() {
                    scatter_direction = hit.normal;
                }
                let scattered = Ray {
                    orig: hit.p,
                    dir: scatter_direction,
                };
                let attenuation = *albedo;
                Some((scattered, attenuation))
            }
            Material::Metal { albedo, fuzz } => {
                let v = ray_in.dir.unit();
                let reflected = reflect(&v, &hit.normal);
                let scattered = Ray {
                    orig: hit.p,
                    dir: reflected + *fuzz * Vec3::random_in_unit_sphere(),
                };
                if scattered.dir.dot(&hit.normal) > 0.0 {
                    let attenuation = *albedo;
                    Some((scattered, attenuation))
                } else {
                    None
                }
            }
            Material::Dielectric { ir } => {
                let attenuation = Color::from([1.0, 1.0, 1.0]);
                let refraction_ratio = match hit.face {
                    Face::Front => 1.0 / ir,
                    Face::Back => *ir,
                };
                let unit_direction = ray_in.dir.unit();
                let cos_theta = (-unit_direction.dot(&hit.normal)).min(1.0);
                let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

                let cannot_refract = refraction_ratio * sin_theta > 1.0;
                let dir = if cannot_refract
                    || reflectance(cos_theta, refraction_ratio) > rand::random()
                {
                    reflect(&unit_direction, &hit.normal)
                } else {
                    refract(&unit_direction, &hit.normal, refraction_ratio)
                };

                let scattered = Ray { orig: hit.p, dir };
                Some((scattered, attenuation))
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct Sphere<'a> {
    pub(crate) center: Point3,
    pub(crate) radius: f64,
    pub(crate) material: &'a Material,
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

impl<'a> Hittable for Sphere<'a> {
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
        Some(HitRecord::new(p, outward_normal, root, ray, self.material))
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
