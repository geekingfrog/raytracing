use eframe::egui::{self, style::Spacing};
use egui::{ColorImage, Rgba};
use egui_extras::RetainedImage;

#[macro_use]
extern crate auto_ops;

mod ray;
mod scene;
mod vec3;

use ray::Ray;
use scene::Scene;
use vec3::{Color, Point3, Vec3};

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Download and show an image with eframe/egui",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

#[derive(Default)]
struct MyApp {
    display: Option<(Scene, RetainedImage)>,
    last_size: egui::Vec2,
}

fn gen_scene(ui: &egui::Ui) -> Scene {
    let size = ui.available_size();
    let aspect_ratio = if size.y == 0.0 { 0.0 } else { size.x / size.y };

    let viewport_height = 2.0;
    let focal_length = 1.0;
    Scene::new(
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
                let scene = gen_scene(ui);
                let img = gen_image(&scene);
                (scene, img)
            });

            if self.last_size != size {
                let scene = gen_scene(ui);
                let i = gen_image(&scene);
                *img = (scene, i);
            };

            img.1.show(ui);
            self.last_size = size;
        });
    }
}

fn ray_color(ray: &Ray) -> Color {
    if hit_sphere(&Vec3::from([0.0, 0.0, -1.0]), 0.5, ray) {
        return Color::from([1.0, 0.0, 0.0]);
    }
    let unit_direction = ray.dir.unit();
    let t = 0.5 * (unit_direction.y + 1.0);
    (1.0 - t) * Color::from([1.0, 1.0, 1.0]) + t * Color::from([0.5, 0.7, 1.0])
}

fn gen_image(scene: &Scene) -> RetainedImage {
    let size = [scene.image_width, scene.image_height];
    let width = size[0];
    let height = size[1];
    println!("gen image {width}x{height}");
    let pixels: Vec<egui::Color32> = (0..height)
        .into_iter()
        .rev()
        .flat_map(|j| (0..width).into_iter().map(move |i| (i, j)))
        .map(|(i, j)| {
            let u = (i as f64) / ((width - 1) as f64);
            let v = (j as f64) / ((height - 1) as f64);
            let dir =
                scene.lower_left_corner + u * scene.horizontal + v * scene.vertical - scene.origin;
            let ray = Ray {
                orig: scene.origin,
                dir,
            };

            ray_color(&ray).into()
        })
        .collect();

    let image = ColorImage { size, pixels };

    RetainedImage::from_color_image("coucoutest", image)
}

fn hit_sphere(center: &Point3, radius: f64, ray: &Ray) -> bool {
    let oc = ray.orig - center;
    let a = ray.dir.dot(&ray.dir);
    let b = 2.0 * oc.dot(&ray.dir);
    let c = oc.dot(&oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;

    discriminant > 0.0
}
