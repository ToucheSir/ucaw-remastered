extern crate piston_window;

use piston_window::*;

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Hello Piston!", [1280, 720])
        .exit_on_esc(true)
        .build()
        .unwrap();

    const WHITE: [f32; 4] = [1.0; 4];
    const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
    const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

    let mut rotation: f64 = 0.0;
    let square = rectangle::square(0.0, 0.0, 50.0);

    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {
            clear(WHITE, g);
             
            line(BLACK, 1.0, [0.0, 360.0, 1280.0, 360.0], c.transform, g);

            let transform = c.transform.trans(640.0, 360.0 + 50.0 * rotation.sin())
                                       .rot_rad(rotation)
                                       .trans(-25.0, -25.0);
            rectangle(RED, square, transform, g);
        });

        if let Some(args) = e.update_args() {
            rotation += 2.0 * args.dt;
        }
    }
}
