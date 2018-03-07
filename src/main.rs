#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::io::prelude::*;

extern crate find_folder;
extern crate nalgebra as na;
extern crate ncollide;
extern crate nphysics2d;
extern crate piston_window;

use piston_window::*;
use std::cell::Ref;
use std::rc::Rc;
use na::{Point2, Translation2, Vector2};
use nphysics2d::world::World;
use nphysics2d::math::Orientation;
use nphysics2d::object::{RigidBody, RigidBodyHandle};
use ncollide::shape::ConvexHull;
use std::thread;

struct Sprite<I: ImageSize> {
    texture: Rc<I>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    frames: u32,
    frame_interval: f32,
    accum_dt: f64,
    current_frame: u32,
}

impl<I: ImageSize> Sprite<I> {
    fn new(
        texture: Rc<I>,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        frames: u32,
        frame_interval: f32,
    ) -> Self {
        Sprite {
            texture,
            x,
            y,
            width,
            height,
            frames,
            frame_interval,
            current_frame: 0,
            accum_dt: 0.0,
        }
    }

    fn update(&mut self, dt: f64) {
        self.accum_dt += dt;
        if self.accum_dt > self.frame_interval as f64 {
            self.accum_dt = 0.0;
            self.current_frame = (self.current_frame + 1) % self.frames;
        }
    }

    fn draw<G: Graphics<Texture=I>>(&self, transform: math::Matrix2d, g: &mut G) {
        let transform = transform.trans(-(self.width as f64) / 2.0, -(self.height as f64) / 2.0);
        Image::new()
            .src_rect([
                (self.x + self.width * self.current_frame) as f64,
                self.y as f64,
                self.width as f64,
                self.height as f64,
            ])
            .draw(self.texture.as_ref(), &DrawState::default(), transform, g);
    }
}

enum Direction {
    Forward,
    Reverse,
    None,
}

enum Rotation {
    Left,
    Right,
    None,
}

extern crate notify;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct GameConfig {
    drag_coeff: f64,
    surface_area: f64,
    air_density: f64,
    thrust_force: f64,
    max_bank_angle: f64,
}

use std::sync::{Arc, RwLock};

fn read_config_file<P: AsRef<std::path::Path>>(path: P) -> GameConfig {
    let mut file = std::fs::File::open(&path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    toml::from_str(&contents).unwrap()
}

fn main() {
    let config_resource = Arc::new(RwLock::new(read_config_file("config/game.toml")));
    let data = config_resource.clone();
    thread::spawn(move || {
        // Create a channel to receive the events.
        let (tx, rx) = channel();

        // Automatically select the best implementation for your platform.
        // You can also access each implementation directly e.g. INotifyWatcher.
        let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1)).unwrap();

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher
            .watch("config/game.toml", RecursiveMode::NonRecursive)
            .unwrap();

        loop {
            match rx.recv() {
                Ok(event) => {
                    println!("{:?}", event);
                    if let notify::DebouncedEvent::Write(path) = event {
                        println!("reloading file");
                        let mut config = data.write().unwrap();
                        *config = read_config_file(&path);
                    }
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });

    let mut window: PistonWindow = WindowSettings::new("Hello Piston!", [1280, 720])
        .exit_on_esc(true)
        .build()
        .unwrap();

    const WHITE: [f32; 4] = [1.0; 4];
    const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
    const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
    const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
    const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

    const METRES_TO_PIXELS: f64 = 4.0;

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets")
        .unwrap();

    let test_img = assets.join("sprites").join("units").join("unit1.png");

    let test_img = Texture::from_path(
        &mut window.factory,
        &test_img,
        Flip::None,
        &TextureSettings::new(),
    ).unwrap();

    let mut cursor: Vector2<f64> = Vector2::new(40.0, 360.0);

    let pos: Vector2<f64> = Vector2::new(20.0, 360.0) / METRES_TO_PIXELS;

    let mut dir = Direction::None;
    let mut rot = Rotation::None;

    let mut world = World::new();

    let points: Vec<Point2<f64>> = vec![
        Point2::new(-4.0, 0.0),
        Point2::new(4.0, -4.0),
        Point2::new(4.0, 4.0),
    ];
    let shape = ConvexHull::new(points);
    // let mut ship = RigidBody::new_dynamic(Ball::new(3.0), 1.0, 0.3, 0.1);
    let mut ship = RigidBody::new_dynamic(shape, 1.0, 0.3, 0.1);
    ship.append_translation(&Translation2::from_vector(pos));
    // ship.prepend_rotation(&na::UnitComplex::new(std::f64::consts::PI / 2.0));
    let ship = world.add_rigid_body(ship);
    let mut sprite = Sprite::new(Rc::new(test_img), 0, 0, 32, 32, 6, 0.1);
    let sprite_width = sprite.width as f64;
    let sprite_height = sprite.height as f64;
    // let square = rectangle::rectangle_by_corners(0.0, 0.0, sprite_width, sprite_height);

    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {
            clear(WHITE, g);

            line(BLACK, 1.0, [0.0, 360.0, 1280.0, 360.0], c.transform, g);
            let bship = ship.borrow();

            let pos = bship.position();
            let translation = METRES_TO_PIXELS * pos.translation.vector;
            let rotation = pos.rotation.angle(); // + std::f64::consts::PI / 2.0;
            let transform = c.transform
                .trans(translation.x, translation.y)
                .rot_rad(rotation);

            let shape = bship.shape().as_shape::<ConvexHull<Point2<f64>>>().unwrap();
            let points = shape
                .points()
                .into_iter()
                .map(|p| (p.coords * METRES_TO_PIXELS).into())
                .collect::<Vec<[f64; 2]>>();
            polygon(RED, points.as_ref(), transform, g);
            let vel = bship.lin_vel() * METRES_TO_PIXELS;
            let acc = bship.lin_acc() * METRES_TO_PIXELS;
            Line::new(BLACK, 2.0).draw(
                [0.0, 0.0, vel.x, vel.y],
                &DrawState::default(),
                transform,
                g,
            );
            Line::new(GREEN, 2.0).draw(
                [0.0, 0.0, acc.x, acc.y],
                &DrawState::default(),
                transform,
                g,
            );
            sprite.draw(transform, g);
        });

        e.update(|args| {
            let config = config_resource.read().unwrap();
            sprite.update(args.dt);
            world.step(args.dt);

            let mut bship = ship.borrow_mut();
            bship.clear_forces();

            let mut dir: Vector2<f64> = cursor - bship.position().translation.vector;
            let rotation = bship.position().rotation;
            let heading = Vector2::new(rotation.cos_angle(), rotation.sin_angle());
            bship.append_lin_force(heading * config.thrust_force);

            let lin_vel = bship.lin_vel();

            let mut angle_between = heading.y.atan2(heading.x) - dir.y.atan2(dir.x);
            if angle_between.abs() > std::f64::consts::PI {
                angle_between = angle_between - 2.0 * std::f64::consts::PI * angle_between.signum();
            }
            if lin_vel.norm() > 0.0 {
                let av = -9.8 * config.max_bank_angle.to_radians().tan() / lin_vel.norm();

                bship.set_ang_vel(na::Vector1::new(av * angle_between.signum()));
            }

            // calculate drag
            let c_d = config.drag_coeff;
            let s_a = config.surface_area;
            // fluid density of air
            let p = config.air_density;
            let facing_dir = Vector2::new(-lin_vel.x.signum(), -lin_vel.y.signum());
            let drag_force =
                0.5 * p * c_d * s_a * lin_vel.component_mul(&lin_vel).component_mul(&facing_dir);
            bship.append_lin_force(drag_force);
        });

        e.mouse_cursor(|x, y| {
            let pos = match window.get_position() {
                Some(p) => p,
                None => Position { x: 0, y: 0 }
            };
            cursor = Vector2::new(x - pos.x as f64, y - pos.y as f64) / METRES_TO_PIXELS;
        });

        e.press(|button| match button {
            Button::Keyboard(key) => match key {
                Key::W => {
                    dir = Direction::Forward;
                }
                Key::S => {
                    dir = Direction::Reverse;
                }
                Key::A => {
                    rot = Rotation::Left;
                }
                Key::D => {
                    rot = Rotation::Right;
                }
                _ => {}
            },
            _ => {}
        });
        e.release(|button| match button {
            Button::Keyboard(key) => match key {
                Key::W | Key::S => {
                    dir = Direction::None;
                }
                Key::A | Key::D => {
                    rot = Rotation::None;
                }
                _ => {}
            },
            _ => {}
        });
    }
}
