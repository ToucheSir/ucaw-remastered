extern crate piston_window;
extern crate find_folder;
extern crate nphysics2d;
extern crate ncollide;
extern crate nalgebra as na;

use piston_window::*;
use std::rc::Rc;
use na::{Vector2, Translation2};
use nphysics2d::world::World;
use nphysics2d::math::{Vector};
use nphysics2d::object::RigidBody;
use nphysics2d::integration::euler;
use ncollide::shape::Ball;

struct Sprite<I: ImageSize> {
    texture: Rc<I>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    frames: u32,
    frame_interval: f32,
    accum_dt: f64,
    current_frame: u32
}

impl <I: ImageSize> Sprite<I> {
    fn new(texture: Rc<I>, x: u32, y: u32, width: u32, height: u32, frames: u32, frame_interval: f32) -> Self {
        Sprite {
            texture,
            x, y, width, height,
            frames, frame_interval,
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
        Image::new().src_rect([
            (self.x + self.width * self.current_frame) as f64, 
            self.y as f64, self.width as f64, self.height as f64
        ]).draw(self.texture.as_ref(), &DrawState::default(), transform, g);
    }
}

enum Direction { Forward, Reverse, None }
enum Rotation { Left, Right, None }

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Hello Piston!", [1280, 720])
        .exit_on_esc(true)
        .build()
        .unwrap();

    const WHITE: [f32; 4] = [1.0; 4];
    const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
    const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();

    let test_img = assets.join("sprites").join("units").join("unit1.png");
    // let test_img = assets.join("sprites").join("units").join("unit8.png");
    let test_img = Texture::from_path(
        &mut window.factory,
        &test_img,
        Flip::None,
        &TextureSettings::new()
    ).unwrap();
    
    let pos: math::Vec2d = [20.0, 360.0];
    let mut dir = Direction::None;
    let mut rot = Rotation::None;

    let mut world = World::new();
    let mut ship = RigidBody::new_dynamic(Ball::new(16.0), 1.0, 0.3, 0.01);
    ship.append_translation(&Translation2::new(pos[0], pos[1]));
    let ship = world.add_rigid_body(ship);

    let mut sprite = Sprite::new(Rc::new(test_img), 0, 0, 32, 32, 12, 0.1);
    // let mut sprite = Sprite::new(Rc::new(test_img), 0, 0, 88, 68, 6, 0.15);
    let sprite_width = sprite.width as f64;
    let sprite_height = sprite.height as f64;
    // let square = rectangle::rectangle_by_corners(0.0, 0.0, sprite_width, sprite_height);
    

    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {
            clear(WHITE, g);
             
            line(BLACK, 1.0, [0.0, 360.0, 1280.0, 360.0], c.transform, g);
            let bship = ship.borrow();
            let pos = bship.position();
            let translation = pos.translation.vector;
            let rotation = pos.rotation.angle();
            let transform = c.transform.trans(translation.x, translation.y)
                .rot_rad(rotation);
            // rectangle(RED, square, transform, g);
            // image(&test_img, transform, g);
            sprite.draw(transform, g);
        });

        e.update(|args| {
            sprite.update(args.dt);
            world.step(args.dt);
            let mut bship = ship.borrow_mut();
            bship.clear_forces();
            // match dir {
            //     Direction::Forward => bship.append_lin_force(Vector::from([1.0, 1.0])),
            //     Direction::Reverse => bship.append_lin_force(Vector::from([-1.0, -1.0])),
            //     Direction::None => {}
            // }
            let mut dir_vec = Vector::from(match dir {
                Direction::Forward => [4000.0, 0.0],
                Direction::Reverse => [-4000.0, 0.0],
                Direction::None => [0.0, 0.0]
            });
            bship.position().rotation.rotate(&mut dir_vec);
            bship.append_lin_force(dir_vec);
            
            // calculate drag
            let lin_vel = bship.lin_vel();
            let Cd = 0.05;
            let A = 1.5;
            // fluid density of air
            let p = 1.25;
            let drag_force = -0.5 * p * Cd * A * (lin_vel.component_mul(&lin_vel));
            bship.append_lin_force(drag_force)

            // bship.append_lin_force(Vector::from(math::mul([rotation.cos(), rotation.sin()], );
            // match rot {
            //     Direction::Left => [1.0, 1.0],
            //     Direction::Right => [-1.0, -1.0],
            //     Direction::None => {}
            // }
            // let velocity = math::mul([rotation.cos(), rotation.sin()], match dir {
            //     Direction::Forward => [1.0, 1.0],
            //     Direction::Reverse => [-1.0, -1.0],
            //     Direction::None => [0.0, 0.0]
            // });
            // pos = math::add(pos, velocity);
        });

        e.press(|button| {
            match button {
                Button::Keyboard(key) => {
                    match key {
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
                    }
                }
                _ => {}
            }
        });
        e.release(|button| {
            match button {
                Button::Keyboard(key) => {
                    match key {
                        Key::W | Key::S => {
                            dir = Direction::None;
                        }
                        Key::A | Key::D => {
                            rot = Rotation::None;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        });
    }
}
