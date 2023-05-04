use std::{
    iter,
    ops::Deref,
    sync::{
        atomic::{AtomicU64, AtomicUsize},
        mpsc, Arc, Mutex,
    },
    thread,
    time::Duration,
};

use crossbeam::queue::ArrayQueue;
use eframe::egui;
use egui::ColorImage;
use egui_extras::RetainedImage;
use material::{Material, Sphere};
use rand::{distributions::Uniform, random, thread_rng, Rng};
use rayon::prelude::*;

mod camera;
mod material;
mod ray;
mod vec3;

use camera::Camera;
use ray::{HitRecord, Hittable, Ray};
use vec3::{Color, Vec3};

// put some globals for now
/// how many ray per pixels (and its neighborhood)
const SAMPLES_PER_PIXEL: usize = 2; // 50;

/// how many maximum bounce for rays before we give up and return black
const MAX_DEPTH: usize = 3; // 40;

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

    // let blue_red = vec![
    //     Material::Lambertian {
    //         albedo: Color::from([0.0, 0.0, 1.0]),
    //     },
    //     Material::Lambertian {
    //         albedo: Color::from([1.0, 0.0, 0.0]),
    //     },
    // ];
    // let r = (std::f64::consts::PI / 4.0).cos();

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

    let n_worker = std::env::var("NUM_WORKER")
        .ok()
        .and_then(|r| usize::from_str_radix(&r, 10).ok())
        .unwrap_or(6);

    let (result_sender, receiver) = mpsc::channel();
    let (command_senders, command_receivers): (Vec<_>, Vec<_>) =
        (0..n_worker).into_iter().map(|_| mpsc::channel()).unzip();

    let world = Arc::new(World::new_random());
    let app = MyApp {
        world: Arc::clone(&world),
        display: None,
        last_size: egui::Vec2::default(),
        ctx: None,
        current_gen: 0,
        commands: command_senders,
        result_channel: receiver,
        state: AppState::Starting,
    };

    for (i, receiver) in command_receivers.into_iter().enumerate() {
        let result_channel = result_sender.clone();
        let world = Arc::clone(&world);
        thread::spawn(move || {
            let mut w = Worker {
                id: i,
                world,
                command_chan: receiver,
                result_channel,
            };
            w.start()
        });
    }

    let options = eframe::NativeOptions::default();
    eframe::run_native("raaaaaaayz", options, Box::new(|_cc| Box::new(app)))
        .expect("eframe app crashed");
    println!("all done");
    Ok(())
}

struct Worker {
    id: usize,
    world: Arc<World>,
    // spec: WorkerSpec,

    // whenever the image to generate changes (change of size or camera)
    // the new specs for stuff to compute are sent there
    command_chan: mpsc::Receiver<WorkerSpec>,

    // computed pixels are sent there: (generation, coordinate (x,y), color)
    result_channel: mpsc::Sender<(usize, (usize, usize), egui::Color32)>,
}

#[derive(Debug)]
struct WorkerSpec {
    generation: usize,
    // TODO: maybe transform that into a Vec of rects so that the load
    // is spread a bit more evenly between different workers.
    /// generate pixels for this range (min_x, min_y), (max_x, max_y)
    range: ((usize, usize), (usize, usize)),
    camera: Camera,
}

impl Worker {
    fn start(&mut self) {
        loop {
            let spec = self
                .command_chan
                .try_iter()
                .last()
                .or_else(|| self.command_chan.recv().ok());

            let spec = match spec {
                Some(s) => s,
                None => {
                    // spec channel has been disconnected, so we also stop
                    println!("Worker {} stopping", self.id);
                    return ();
                }
            };

            let ((x0, y0), (x1, y1)) = spec.range;
            for i in x0..x1 {
                for j in y0..y1 {
                    let mut color = Color::default();
                    for _ in 0..SAMPLES_PER_PIXEL {
                        let u =
                            (i as f64 + random::<f64>()) / ((spec.camera.image_width - 1) as f64);
                        let v =
                            (j as f64 + random::<f64>()) / ((spec.camera.image_height - 1) as f64);
                        let ray = spec.camera.get_ray(u, v);
                        let w: &'_ World = &self.world;
                        color += ray_color(w, &ray, 0);
                    }

                    let scale = 1.0 / SAMPLES_PER_PIXEL as f64;
                    color = (color * scale).sqrt();
                    let color: egui::Color32 = color.into();
                    let result = (spec.generation, (i, j), color);
                    self.result_channel.send(result).unwrap();
                }
            }
        }
    }
}

struct MyApp {
    world: Arc<World>,
    display: Option<(Camera, RetainedImage)>,
    last_size: egui::Vec2,
    ctx: Option<egui::Context>,
    current_gen: usize,
    commands: Vec<mpsc::Sender<WorkerSpec>>,
    result_channel: mpsc::Receiver<(usize, (usize, usize), egui::Color32)>,
    state: AppState,
}

enum AppState {
    Starting,
    Computing {
        camera: Camera,
        pixels: Vec<egui::Color32>,
        prev_image: RetainedImage,
    },
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

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame = egui::containers::Frame::none();
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            println!("size: {:?}", ui.available_size());
            match &mut self.state {
                AppState::Starting => {
                    let ctx2 = ctx.clone();
                    thread::spawn(move || loop {
                        thread::sleep(Duration::from_millis(500));
                        ctx2.request_repaint();
                    });

                    println!("starting for size {:?}", ui.available_size());
                    // let camera = gen_camera(&ui.available_size());
                    let camera = gen_camera(&[561.9,294.4].into());
                    let n = camera.image_width * camera.image_height;
                    let pixels: Vec<egui::Color32> = (0..n)
                        .into_iter()
                        .map(|_| egui::Color32::LIGHT_RED)
                        .collect();
                    let image = ColorImage {
                        size: [camera.image_width, camera.image_height],
                        pixels: pixels.clone(),
                    };
                    let image = RetainedImage::from_color_image("coucoustart", image);
                    image.show(ui);

                    let band_width = camera.image_width / self.commands.len();
                    let band_offset = camera.image_width % self.commands.len();
                    for (i, chan) in self.commands.iter().enumerate() {
                        let (x0, x1) = if i == 0 {
                            (0, band_width + band_offset)
                        } else {
                            let x = band_offset + band_width * i;
                            (x, x + band_width)
                        };

                        let spec = WorkerSpec {
                            generation: self.current_gen,
                            range: ((x0, 0), (x1, camera.image_height)),
                            camera: camera.clone(),
                        };
                        chan.send(spec).unwrap();
                    }

                    self.state = AppState::Computing {
                        camera,
                        pixels,
                        prev_image: image,
                    };
                }
                AppState::Computing {
                    camera,
                    pixels,
                    prev_image,
                } => {
                    for (gen, (x, y), color) in self.result_channel.try_iter() {
                        if gen != self.current_gen {
                            continue;
                        }
                        pixels[camera.image_width * y + x] = color;
                    }
                    let image = ColorImage {
                        size: [camera.image_width, camera.image_height],
                        pixels: pixels.clone(),
                    };
                    let image = RetainedImage::from_color_image("", image);
                    *prev_image = image;
                    prev_image.show(ui);
                    ()
                }
            }

            // let size = ui.available_size();
            // if self.last_size == egui::Vec2::ZERO {
            //     self.last_size = size;
            // }

            // if self.ctx.is_none() {
            //     self.ctx = Some(ctx.clone());
            // };

            // let img = self.display.get_or_insert_with(|| {
            //     let camera = gen_camera(&ui.available_size());
            //     let img = gen_image(&self.world, &camera);
            //     (camera, img)
            // });

            // if self.last_size != size {
            //     let camera = gen_camera(&ui.available_size());
            //     let i = gen_image(&self.world, &camera);
            //     *img = (camera, i);
            // };

            // img.1.show(ui);
            // self.last_size = size;
        });
    }
}

fn ray_color<T>(world: &T, ray: &Ray, depth: usize) -> Color
where
    T: Hittable,
{
    if depth >= MAX_DEPTH {
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
                    attenuation * ray_color(world, &scattered, depth + 1)
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

fn gen_image(world: &World, camera: &Camera) -> RetainedImage {
    let start = std::time::Instant::now();
    let size = [camera.image_width, camera.image_height];
    let width = size[0];
    let height = size[1];

    println!("gen image {width}x{height} ({SAMPLES_PER_PIXEL})");
    let pixels: Vec<egui::Color32> = (0..height)
        .into_par_iter()
        .rev()
        .flat_map_iter(|j| (0..width).into_iter().map(move |i| (i, j)))
        .map(|(i, j)| {
            // if i == 0 {
            //     println!("line {}", j);
            // }

            let mut color = Color::default();
            for _ in 0..SAMPLES_PER_PIXEL {
                let u = (i as f64 + random::<f64>()) / ((width - 1) as f64);
                let v = (j as f64 + random::<f64>()) / ((height - 1) as f64);
                let ray = camera.get_ray(u, v);
                color += ray_color(&world, &ray, 0);
            }

            let scale = 1.0 / SAMPLES_PER_PIXEL as f64;
            color = (color * scale).sqrt();
            color.into()
        })
        .collect();

    let image = ColorImage { size, pixels };

    let image = RetainedImage::from_color_image("coucoutest", image);
    println!("took {}ms", start.elapsed().as_millis());
    image
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
