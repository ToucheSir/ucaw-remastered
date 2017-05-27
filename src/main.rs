extern crate piston_window;
extern crate find_folder;

use piston_window::*;
use std::rc::Rc;

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
            texture: texture,
            x: x,
            y: y,
            width: width,
            height: height,
            frames: frames,
            current_frame: 0,
            frame_interval: frame_interval,
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
        Image::new().src_rect([
            (self.x + self.width * self.current_frame) as f64, 
            self.y as f64, self.width as f64, self.height as f64
        ]).draw(self.texture.as_ref(), &DrawState::default(), transform, g);
    }
}

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

    let mut sprite = Sprite::new(Rc::new(test_img), 0, 0, 32, 32, 12, 0.1);
    // let mut sprite = Sprite::new(Rc::new(test_img), 0, 0, 88, 68, 6, 0.15);
    let sprite_width = sprite.width as f64;
    let sprite_height = sprite.height as f64;
    let square = rectangle::rectangle_by_corners(0.0, 0.0, sprite_width, sprite_height);
    
    let mut pos: math::Vec2d = [0.0, 360.0];

    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {
            clear(WHITE, g);
             
            line(BLACK, 1.0, [0.0, 360.0, 1280.0, 360.0], c.transform, g);

            let transform = c.transform.trans(pos[0], pos[1]);
            rectangle(RED, square, transform, g);
            // image(&test_img, transform, g);
            sprite.draw(transform, g);
        });

        e.update(|args| {
            sprite.update(args.dt);
        });

        e.press(|button| {
            match button {
                Button::Keyboard(key) => {
                    match key {
                        Key::W => {
                            pos[1] -= 1.0;
                        }
                        Key::S => {
                            pos[1] += 1.0;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        });
    }
}
