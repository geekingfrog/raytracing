use crate::egui::Color32;
use std::fmt::Display;

use auto_ops::*;
use rand::{distributions::Uniform, random, thread_rng, Rng};

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Vec3 {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
}

pub(crate) const ZERO: Vec3 = Vec3 {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};

/// 3D point
pub(crate) type Point3 = Vec3;

/// RGB color
pub(crate) type Color = Vec3;

impl From<[f64; 3]> for Vec3 {
    fn from(x: [f64; 3]) -> Self {
        Vec3 {
            x: x[0],
            y: x[1],
            z: x[2],
        }
    }
}

impl From<Vec3> for Color32 {
    fn from(c: Vec3) -> Self {
        Color32::from_rgb(
            (c.x * 255.999) as _,
            (c.y * 255.999) as _,
            (c.z * 255.999) as _,
        )
    }
}

impl Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{},{})", self.x, self.y, self.z)
    }
}

impl Vec3 {
    pub(crate) fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub(crate) fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub(crate) fn dot(&self, v: &Vec3) -> f64 {
        let u = self;
        u.x * v.x + u.y * v.y + u.z * v.z
    }

    pub(crate) fn cross(&self, v: &Vec3) -> Vec3 {
        let u = self;
        Vec3 {
            x: u.y * v.z - u.z * v.y,
            y: u.z * v.x - u.x * v.z,
            z: u.x * v.y - u.y * v.x,
        }
    }

    pub(crate) fn unit(&self) -> Self {
        self / self.length()
    }

    pub(crate) fn random() -> Self {
        Vec3 {
            x: random(),
            y: random(),
            z: random(),
        }
    }

    pub(crate) fn random_range(min: f64, max: f64) -> Self {
        let mut rng = thread_rng();
        let d = Uniform::new(min, max);
        Vec3 {
            x: rng.sample(d),
            y: rng.sample(d),
            z: rng.sample(d),
        }
    }

    pub(crate) fn random_in_unit_sphere() -> Self {
        loop {
            let p = Self::random_range(-1.0, 1.0);
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }
}

impl_op_ex!(+ |a: &Vec3, b: &Vec3| -> Vec3 {
    Vec3{
            x: a.x + b.x,
            y: a.y + b.y,
            z: a.z + b.z,
    }
});

impl_op_ex!(-|a: &Vec3, b: &Vec3| -> Vec3 { a + (-b) });

impl_op_ex!(+= |a: &mut Vec3, b: &Vec3| {
    a.x += b.x;
    a.y += b.y;
    a.z += b.z;
});

impl_op_ex!(*|a: &Vec3, b: &Vec3| -> Vec3 {
    Vec3 {
        x: a.x * b.x,
        y: a.y * b.y,
        z: a.z * b.z,
    }
});

impl_op_ex!(*= |a: &mut Vec3, b: &Vec3| {
    a.x *= b.x;
    a.y *= b.y;
    a.z *= b.z;
});

impl_op_ex!(/|a: &Vec3, b: &Vec3| -> Vec3 {
    Vec3 {
        x: a.x / b.x,
        y: a.y / b.y,
        z: a.z / b.z,
    }
});

impl_op_ex_commutative!(*|a: &Vec3, t: f64| -> Vec3 {
    Vec3 {
        x: a.x * t,
        y: a.y * t,
        z: a.z * t,
    }
});

impl_op_ex_commutative!(/ |a: &Vec3, t: f64| -> Vec3 {
    (1.0/t) * a
});

impl_op_ex!(/= |a: &mut Vec3, b: &Vec3| {
    a.x /= b.x;
    a.y /= b.y;
    a.z /= b.z;
});

impl_op_ex!(-|a: &Vec3| -> Vec3 {
    Vec3 {
        x: -a.x,
        y: -a.y,
        z: -a.z,
    }
});
