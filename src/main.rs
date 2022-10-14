use eframe::egui;
use egui::ColorImage;
use egui_extras::RetainedImage;
use rand::random;

mod ray;
mod camera;
mod vec3;

use ray::Ray;
use camera::Camera;
use vec3::{Color, Point3, Vec3};

#[derive(Default)]
struct World(Vec<Sphere>);

fn main() {
    let options = eframe::NativeOptions::default();
    let world = World(vec![
        Sphere {
            center: Vec3::from([0.0, 0.0, -1.0]),
            radius: 0.5,
        },
        Sphere {
            center: Vec3::from([0.0, -100.5, -1.0]),
            radius: 100.0,
        },
    ]);

    let app = MyApp {
        world,
        ..Default::default()
    };

    eframe::run_native("raaaaaaayz", options, Box::new(|_cc| Box::new(app)));
}

#[derive(Default)]
struct MyApp {
    world: World,
    display: Option<(Camera, RetainedImage)>,
    last_size: egui::Vec2,
}

fn gen_camera(size: &egui::Vec2) -> Camera {
    let aspect_ratio = if size.y == 0.0 { 0.0 } else { size.x / size.y };

    let viewport_height = 2.0;
    let focal_length = 1.0;
    Camera::new(
        aspect_ratio.into(),
        size.x as usize,
        viewport_height,
        focal_length,
        vec3::ZERO,
    )
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame = egui::containers::Frame::none();
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            let size = ui.available_size();
            if self.last_size == egui::Vec2::ZERO {
                self.last_size = size;
            }

            let img = self.display.get_or_insert_with(|| {
                let camera = gen_camera(&ui.available_size());
                let img = gen_image(&self.world, &camera);
                (camera, img)
            });

            if self.last_size != size {
                let camera = gen_camera(&ui.available_size());
                let i = gen_image(&self.world, &camera);
                *img = (camera, i);
            };

            img.1.show(ui);
            self.last_size = size;
        });
    }
}

fn ray_color<T>(world: T, ray: &Ray) -> Color
where
    T: Hittable,
{
    match world.hit(ray, 0.0, f64::INFINITY) {
        Some(hit) => {
            0.5 * (hit.normal + Color::from([1.0, 1.0, 1.0]))
        }
        None => {
            let unit_direction = ray.dir.unit();
            let t = 0.5 * (unit_direction.y + 1.0);
            (1.0 - t) * Color::from([1.0, 1.0, 1.0]) + t * Color::from([0.5, 0.7, 1.0])
        }
    }
}

fn gen_image(world: &World, camera: &Camera) -> RetainedImage {
    let start = std::time::Instant::now();
    let size = [camera.image_width, camera.image_height];
    let width = size[0];
    let height = size[1];
    let samples_per_pixel = 100;

    println!("gen image {width}x{height} ({samples_per_pixel})");
    let pixels: Vec<egui::Color32> = (0..height)
        .into_iter()
        .rev()
        .flat_map(|j| (0..width).into_iter().map(move |i| (i, j)))
        .map(|(i, j)| {
            let mut color = Color::default();
            for _ in 0..samples_per_pixel {
                let u = (i as f64 + random::<f64>()) / ((width - 1) as f64);
                let v = (j as f64 + random::<f64>()) / ((height - 1) as f64);
                let ray = camera.get_ray(u, v);
                color += ray_color(world, &ray);
            };

            let scale = 1.0 / samples_per_pixel as f64;
            color = color * scale;
            color.into()
        })
        .collect();

    let image = ColorImage { size, pixels };

    let image = RetainedImage::from_color_image("coucoutest", image);
    println!("took {}ms", start.elapsed().as_millis());
    image
}

#[derive(Debug)]
enum Face {
    Front,
    Back,
}

#[derive(Debug)]
struct HitRecord {
    p: Point3,
    normal: Vec3,
    t: f64,
    face: Face,
}

impl HitRecord {
    fn new(p: Point3, outward_normal: Vec3, t: f64, ray: &Ray) -> Self {
        let (normal, face) = if ray.dir.dot(&outward_normal) > 0.0 {
            (-outward_normal, Face::Back)
        } else {
            (outward_normal, Face::Front)
        };
        Self { p, normal, t, face }
    }
}

trait Hittable {
    fn hit(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<HitRecord>;
}

#[derive(Debug, Default)]
struct Sphere {
    center: Point3,
    radius: f64,
}

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
        Some(HitRecord::new(p, outward_normal, root, ray))
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

impl<'a> Hittable for &'a World {
    fn hit(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<HitRecord> {
        self.0.hit(ray, tmin, tmax)
    }
}
