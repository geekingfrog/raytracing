use std::{
    sync::{mpsc, Arc},
    thread,
    time::{Duration, Instant},
};

use eframe::egui;
use egui::ColorImage;
use egui_extras::RetainedImage;
use material::{Material, Sphere};
use rand::{distributions::Uniform, random, seq::SliceRandom, thread_rng, Rng};
use rayon::prelude::*;

mod camera;
mod material;
mod ray;
mod vec3;

use camera::Camera;
use ray::{HitRecord, Hittable, Ray};
use vec3::{Color, Vec3};

/// how many ray per pixels (and its neighborhood)
const SAMPLES_PER_PIXEL: usize = 50;

/// how many maximum bounce for rays before we give up and return black
const MAX_DEPTH: usize = 40;

struct World {
    spheres: Vec<Sphere>,
}

impl World {
    fn new_random() -> Self {
        let mut spheres = vec![];
        let mut rng = thread_rng();

        let ground_material = Material::Lambertian {
            albedo: Color::from([0.5, 0.5, 0.5]),
        };
        spheres.push(Sphere {
            center: Vec3::from([0, -1000, 0]),
            radius: 1000.0,
            material: ground_material,
        });

        for a in -11..11 {
            for b in -11..11 {
                let center = Vec3::from([
                    a as f64 + 0.9 * random::<f64>(),
                    0.2,
                    b as f64 + 0.9 * random::<f64>(),
                ]);

                if (center - Vec3::from([4.0, 0.2, 0.0])).length() > 0.9 {
                    let choose_mat = random::<f64>();
                    if choose_mat < 0.8 {
                        // diffuse
                        let albedo = Color::random() * Color::random();
                        let material = Material::Lambertian { albedo };
                        spheres.push(Sphere {
                            center,
                            radius: 0.2,
                            material,
                        });
                    } else if choose_mat < 0.95 {
                        // metal
                        let albedo = Color::random_range(0.5, 1.0);
                        let fuzz = rng.sample(Uniform::new(0.0, 0.5));
                        let material = Material::Metal { albedo, fuzz };
                        spheres.push(Sphere {
                            center,
                            radius: 0.2,
                            material,
                        });
                    } else {
                        // glass
                        spheres.push(Sphere {
                            center,
                            radius: 0.2,
                            material: Material::Dielectric { ir: 1.5 },
                        })
                    }
                }
            }
        }

        spheres.push(Sphere {
            center: Vec3::from([0, 1, 0]),
            radius: 1.0,
            material: Material::Dielectric { ir: 1.5 },
        });

        spheres.push(Sphere {
            center: Vec3::from([-4, 1, 0]),
            radius: 1.0,
            material: Material::Lambertian {
                albedo: Color::from([0.5, 0.2, 0.1]),
            },
        });

        spheres.push(Sphere {
            center: Vec3::from([4, 1, 0]),
            radius: 1.0,
            material: Material::Metal {
                albedo: Color::from([0.7, 0.6, 0.5]),
                fuzz: 0.0,
            },
        });

        World { spheres }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let material_ground = Material::Lambertian {
        albedo: Color::from([0.8, 0.8, 0.0]),
    };
    let material_center = Material::Lambertian {
        albedo: Color::from([0.1, 0.2, 0.5]),
    };
    let material_left = Material::Dielectric { ir: 1.5 };
    let material_right = Material::Metal {
        albedo: Color::from([0.8, 0.6, 0.2]),
        fuzz: 0.0,
    };

    let materials = vec![
        material_ground,
        material_center,
        material_left,
        material_right,
    ];

    #[allow(unused_variables)]
    let world = World {
        spheres: vec![
            Sphere {
                center: Vec3::from([0.0, -100.5, -1.0]),
                radius: 100.0,
                material: materials[0].clone(),
            },
            Sphere {
                center: Vec3::from([0.0, 0.0, -1.0]),
                radius: 0.5,
                material: materials[1].clone(),
            },
            Sphere {
                center: Vec3::from([-1.0, 0.0, -1.0]),
                radius: 0.5,
                material: materials[2].clone(),
            },
            Sphere {
                center: Vec3::from([-1.0, 0.0, -1.0]),
                // negative radius for dielectric material (glass) means the normal
                // points inward, which creates a hollow glass sphere
                radius: -0.4,
                material: materials[2].clone(),
            },
            Sphere {
                center: Vec3::from([1.0, 0.0, -1.0]),
                radius: 0.5,
                material: materials[3].clone(),
            },
        ],
    };

    let world = World::new_random();
    let app = MyApp {
        world: Arc::new(world),
        state: AppState::Starting,
    };

    let options = eframe::NativeOptions::default();
    eframe::run_native("raaaaaaayz", options, Box::new(|_cc| Box::new(app)))
        .expect("eframe app crashed");
    println!("all done");
    Ok(())
}

struct BackgroundWorker {
    samples_per_pixel: usize,
    max_depth: usize,
}

impl BackgroundWorker {
    /// given a world and a camera, initiate a background computation
    /// using multiple threads to compute the image.
    /// It returns a channel with (x, y, color)
    /// If that computation is no longer relevant (camera or world changed for exampe)
    /// the receiver should be dropped and the threads will stop shortly after.
    fn start(&self, world: Arc<World>, camera: &Camera) -> mpsc::Receiver<(usize, usize, Color)> {
        let (sender, rx) = mpsc::channel();

        let samples_per_pixel = self.samples_per_pixel;
        let max_depth = self.max_depth;
        let camera = Arc::new(camera.clone());
        thread::spawn(move || {
            let start = Instant::now();
            let mut coords = (0..camera.image_height)
                .into_iter()
                .flat_map(|j| (0..camera.image_width).into_iter().map(move |i| (i, j)))
                .collect::<Vec<_>>();

            // shuffling the coords make the image appears in a more uniform manner
            // which I prefer
            coords.shuffle(&mut thread_rng());

            for _ in 0..samples_per_pixel {
                let sender = sender.clone();
                let res = coords
                    .par_iter()
                    .try_for_each_with(sender, |sender, (i, j)| {
                        let u = (*i as f64 + random::<f64>()) / ((camera.image_width - 1) as f64);
                        let v = (*j as f64 + random::<f64>()) / ((camera.image_height - 1) as f64);
                        let ray = camera.get_ray(u, v);
                        sender.send((*i, *j, ray_color(&world, max_depth, &ray, 0)))
                    });

                // ignore the error since the only error we can get is because
                // the channel to send the result has been closed. In this case
                // this thread should just stop and die quietly, what it is
                // computing is no longer relevant (typically, window got resized)
                if res.is_err() {
                    return;
                }
            }

            let dur = start.elapsed().as_millis();
            println!(
                "image took {}ms with {} samples per pixels with at most {} reflections",
                dur, samples_per_pixel, max_depth
            );
        });
        rx
    }
}

struct MyApp {
    world: Arc<World>,
    state: AppState,
}

enum AppState {
    Starting,
    Computing {
        img_buffer: ImageBuffer,
        prev_size: egui::Vec2,
        prev_image: RetainedImage,
        result_channel: mpsc::Receiver<(usize, usize, Color)>,
    },
}

struct ImageBuffer {
    width: usize,
    height: usize,
    pixels: Vec<(Color, usize)>,
}

impl ImageBuffer {
    fn new(width: usize, height: usize) -> Self {
        let pixels = std::iter::repeat((Color::default(), 0))
            .take(width * height)
            .collect();
        Self {
            width,
            height,
            pixels,
        }
    }

    fn update_at(&mut self, result: (usize, usize, Color)) {
        let (x, y, col) = result;
        // reverse the y axis because the internal image representation
        // has its y axis pointing downward while our own axis is upward
        let y = self.height - y - 1;
        let idx = self.width * y + x;
        let (c, n) = self.pixels[idx];
        self.pixels[idx] = (c + col, n + 1);
    }

    fn to_retained_image(&self) -> RetainedImage {
        let pixels = self
            .pixels
            .iter()
            .map(|(col, n)| {
                let scale = 1.0 / (*n as f64);
                let color = (col * scale).sqrt();
                let color: egui::Color32 = color.into();
                color
            })
            .collect::<Vec<_>>();
        let img = ColorImage {
            size: [self.width, self.height],
            pixels,
        };
        RetainedImage::from_color_image("", img)
    }
}

fn gen_camera(size: &egui::Vec2) -> Camera {
    let aspect_ratio = if size.y == 0.0 { 0.0 } else { size.x / size.y };

    let focal_length = 1.0;
    let look_from = Vec3::from([13, 2, 3]);
    let look_at = Vec3::from([0, 0, 0]);
    let vup = Vec3::from([0, 1, 0]);
    let aperture = 0.1;
    let dist_to_focus = (look_from - look_at).length();
    // let dist_to_focus = 10.0;
    Camera::new(
        look_from,
        look_at,
        vup,
        20.0,
        aspect_ratio.into(),
        size.x as usize,
        focal_length,
        aperture,
        dist_to_focus,
    )
}

impl MyApp {
    fn start(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let size = ui.available_size();
        let camera = gen_camera(&size);
        println!(
            "{:?} - generating image for {:?}",
            time::OffsetDateTime::now_utc(),
            size
        );

        let img_buffer = ImageBuffer::new(camera.image_width, camera.image_height);
        let image = img_buffer.to_retained_image();
        image.show(ui);

        let spx = std::env::var("SAMPLES_PER_PIXEL")
            .ok()
            .and_then(|r| usize::from_str_radix(&r, 10).ok())
            .unwrap_or(SAMPLES_PER_PIXEL);

        let max_depth = std::env::var("MAX_DEPTH")
            .ok()
            .and_then(|r| usize::from_str_radix(&r, 10).ok())
            .unwrap_or(MAX_DEPTH);

        let bgw = BackgroundWorker {
            samples_per_pixel: spx,
            max_depth,
        };
        let result_channel = bgw.start(Arc::clone(&self.world), &camera);
        ctx.request_repaint_after(Duration::from_millis(32));

        self.state = AppState::Computing {
            img_buffer,
            prev_size: size,
            prev_image: image,
            result_channel,
        };
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame = egui::containers::Frame::none();
        egui::CentralPanel::default()
            .frame(frame)
            .show(ctx, |mut ui| {
                match &mut self.state {
                    AppState::Starting => {
                        self.start(ctx, &mut ui);
                    }
                    AppState::Computing {
                        img_buffer,
                        prev_size,
                        prev_image,
                        result_channel,
                    } => {
                        let size = ui.available_size();
                        if &size != prev_size {
                            *prev_size = size;
                            self.start(ctx, &mut ui);
                            return;
                        };

                        // ensure we keep updating the image even if there's no user
                        // activity (in this case, `update` isn't called)
                        ctx.request_repaint_after(Duration::from_millis(32));

                        for result in result_channel.try_iter() {
                            img_buffer.update_at(result)
                        }
                        let image = img_buffer.to_retained_image();
                        *prev_image = image;
                        prev_image.show(ui);
                        ()
                    }
                }
            });
    }
}

fn ray_color<T>(world: &T, max_depth: usize, ray: &Ray, depth: usize) -> Color
where
    T: Hittable,
{
    if depth >= max_depth {
        return Color::default();
    }

    match world.hit(ray, 0.0001, f64::INFINITY) {
        Some(hit) => {
            // lazy lambertian, which has a distribution of cos³(Φ), with Φ the
            // angle from the normal. That means we prefer reflections closer to
            // the normal, meaning lower probability for rays at grazing angle.
            // let target = hit.p + hit.normal + Vec3::random_in_unit_sphere();

            // lambertian reflection, which has a distribution of cos(Φ)
            // this leads to less pronounced shadows, and lighter spheres.
            // let target = hit.p + hit.normal + Vec3::random_in_unit_sphere();

            // let target = hit.p + Vec3::random_in_hemisphere(&hit.normal);

            match hit.mat.scatter(ray, &hit) {
                Some((scattered, attenuation)) => {
                    attenuation * ray_color(world, max_depth, &scattered, depth + 1)
                }
                None => Color::default(),
            }
            //
            // let r = Ray {
            //     orig: hit.p,
            //     dir: target - hit.p,
            // };
            // 0.5 * ray_color(world, &r, depth + 1)
        }
        None => {
            let unit_direction = ray.dir.unit();
            let t = 0.5 * (unit_direction.y + 1.0);
            (1.0 - t) * Color::from([1, 1, 0]) + t * Color::from([0.5, 0.7, 1.0])
        }
    }
}

impl<'a> Hittable for &'a World {
    fn hit(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<HitRecord> {
        self.spheres.hit(ray, tmin, tmax)
    }
}

impl Hittable for World {
    fn hit(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<HitRecord> {
        self.spheres.hit(ray, tmin, tmax)
    }
}

impl Hittable for Arc<World> {
    fn hit(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<HitRecord> {
        self.spheres.hit(ray, tmin, tmax)
    }
}
