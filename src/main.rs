extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use std::env;
use std::f64::consts::PI;
use std::time::Instant;

use colors_transform::{Color, Hsl, Rgb};
use glutin_window::GlutinWindow as Window;
use graphics::{CircleArc, DrawState, Ellipse, Graphics, Line};
use graphics::types::Matrix2d;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{UpdateArgs, UpdateEvent};
use piston::event_loop::{Events, EventSettings};
use piston::input::{RenderArgs, RenderEvent};
use piston::window::WindowSettings;

mod compile_pngs;

#[derive(Clone, Copy, PartialEq)]
enum Quality {
    // High - 3, Medium - 2, Low - 1
    High = 3,
    Medium = 2,
    Low = 1,
}

#[derive(Clone, Copy)]
enum VaryingType {
    Angle = 0,
    Length1 = 1,
    Length2 = 2,
    Mass1 = 3,
    Mass2 = 4,
}

#[derive(Clone, Copy)]
struct Pendulum {
    r1: f64,
    r2: f64,
    m1: f64,
    m2: f64,
    a1: f64,
    a2: f64,
    a1_v: f64,
    a2_v: f64,
    color: Rgb,
}

#[derive(Clone, Copy)]
struct Settings {
    g: f64,
    max_m1: f64,
    max_m2: f64,
    r1: f64,
    r2: f64,
    // sep: f64,
    mag: f64,
    pend_transp: f64,
    pend_width: f64,
    speed: f64,
    quality: Quality,
    varying: VaryingType,
    // amt_pend: i32,
}

impl Settings {
    fn new() -> Settings {
        Settings {
            g: 0.1,
            max_m1: 10.0,
            max_m2: 10.0,
            r1: 125.0,
            r2: 125.0,
            // sep: 0.1,
            mag: 2.0,
            pend_transp: 0.05,
            pend_width: 1.5,
            speed: 1.0,
            quality: Quality::Low,
            varying: VaryingType::Angle,
            // amt_pend: 50_000,
        }
    }
}

impl Pendulum {
    // Constructor
    fn new(d: f64, d2: f64, r1i: f64, r2i: f64, m1i: f64, m2i: f64, color: Rgb) -> Pendulum {
        Pendulum {
            r1: r1i,
            r2: r2i,
            m1: m1i,
            m2: m2i,
            a1: PI / d,
            a2: PI / d2,
            a1_v: 0.0,
            a2_v: 0.0,
            color,
        }
    }


    fn update_draw(
        mut self,
        settings: Settings,
        transform: Matrix2d,
        gl: &mut GlGraphics,
    ) -> Pendulum {
        let a1 = self.a1;
        let a2 = self.a2;
        let a1_v = self.a1_v;
        let a2_v = self.a2_v;
        let r1 = self.r1;
        let r2 = self.r2;
        let m1 = self.m1;
        let m2 = self.m2;
        let g = settings.g;

        let mut num1 = -g * (2.0 * m1 + m2) * (a1);
        let mut num2 = -m2 * g * (a1 - 2.0 * a2).sin();
        let mut num3 = -2.0 * (a1 - a2).sin() * m2;
        let mut num4 = a2_v * a2_v * r2 + a1_v * a1_v * r1 * (a1 - a2).cos();
        let mut den = r1 * (2.0 * m1 + m2 - m2 * (2.0 * a1 - 2.0 * a2).cos());
        let a1_a = (num1 + num2 + num3 * num4) / den;

        num1 = 2.0 * (a1 - a2).sin();
        num2 = a1_v * a1_v * r1 * (m1 + m2);
        num3 = g * (m1 + m2) * (a1).cos();
        num4 = a2_v * a2_v * r2 * m2 * (a1 - a2).cos();
        den = r2 * (2.0 * m1 + m2 - m2 * (2.0 * a1 - 2.0 * a2).cos());
        let a2_a = (num1 * (num2 + num3 + num4)) / den;

        let x1 = r1 * (a1).sin();
        let y1 = r1 * (a1).cos();

        let x2 = x1 + r2 * (a2).sin();
        let y2 = y1 + r2 * (a2).cos();

        let line_t = Line::new(
            [
                self.color.get_red() as f32 / 255.0,
                self.color.get_blue() as f32 / 255.0,
                self.color.get_green() as f32 / 255.0,
                settings.pend_transp as f32,
            ],
            settings.pend_width,
        );

        let mag = settings.mag;

        line_t.draw(
            [0.0, 0.0, x1 * mag, y1 * mag],
            &DrawState::default(),
            transform,
            gl,
        );
        line_t.draw(
            [x1 * mag, y1 * mag, x2 * mag, y2 * mag],
            &DrawState::default(),
            transform,
            gl,
        );

        if settings.quality == Quality::Medium {
            // Draw a circle at the middle and end of the pend depending on mass
            let circle_t = Ellipse::new([
                self.color.get_red() / 255.0,
                self.color.get_blue() / 255.0,
                self.color.get_green() / 255.0,
                settings.pend_transp as f32,
            ]);
            circle_t.draw(
                [x1 * mag - (m1 / 2.0), y1 * mag - (m1 / 2.0), m1, m1],
                &DrawState::default(),
                transform,
                gl,
            );
            circle_t.draw(
                [x2 * mag - (m2 / 2.0), y2 * mag - (m2 / 2.0), m2, m2],
                &DrawState::default(),
                transform,
                gl,
            );
        }
        else if settings.quality == Quality::High {
            let circle_t = Ellipse::new([
                self.color.get_red() / 255.0,
                self.color.get_blue() / 255.0,
                self.color.get_green() / 255.0,
                settings.pend_transp as f32,
            ]);
            let bl_circle_t = CircleArc::new([0.0, 0.0, 0.0, settings.pend_transp as f32],0.0, 0.0, 2.0 * PI);
            circle_t.draw(
                [x1 * mag - (m1 / 2.0), y1 * mag - (m1 / 2.0), m1, m1],
                &DrawState::default(),
                transform,
                gl,
            );
            circle_t.draw(
                [x2 * mag - (m2 / 2.0), y2 * mag - (m2 / 2.0), m2, m2],
                &DrawState::default(),
                transform,
                gl,
            );
            bl_circle_t.draw(
                [x1 * mag - (m1 / 2.0), y1 * mag - (m1 / 2.0), m1, m1],
                &DrawState::default(),
                transform,
                gl,
            );
            bl_circle_t.draw(
                [x2 * mag - (m2 / 2.0), y2 * mag - (m2 / 2.0), m2, m2],
                &DrawState::default(),
                transform,
                gl,
            );
        }

        self.a1_v += a1_a * settings.speed;
        self.a2_v += a2_a * settings.speed;
        self.a1 += a1_v * settings.speed;
        self.a2 += a2_v * settings.speed;

        self
    }
}

pub struct App {
    gl: GlGraphics,
    pends: Vec<Pendulum>,
    settings: Settings,
    fps_counter: u32,
    last_update: Instant,
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let (mid_x, mid_y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear([0.2, 0.2, 0.2, 1.0], gl);

            for ui in 0..self.pends.len() {
                //((time/0.5) as u8)  {
                let new_transform = c.transform.trans(mid_x, mid_y);

                self.pends[ui] = self.pends[ui].update_draw(self.settings, new_transform, gl);
            }
        });
    }

    // Update function to print the fps to the console.
    fn update(&mut self, args: &UpdateArgs) {
        if (self.fps_counter % 10) == 0 {
            let now = Instant::now();
            let elapsed = now.duration_since(self.last_update);
            let fps = 10.0 / elapsed.as_secs_f64();
            println!("FPS: {}", fps);
            self.last_update = now;
        }
        self.fps_counter += 1;
    }
}

pub fn main() {
    let mut amt_pend: usize = 50_000;
    let mut amt_sep: f64 = 0.1;

    // Parse the command line arguments and add help args which return and print a message.
    let args: Vec<String> = env::args().collect();
    let mut settings = Settings::new();

    // Parse command line arguments
    let mut compile = false;
    let mut compile_frames = 50;
    for i in 0..args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                println!("Usage: {} [OPTIONS]", args[0]);
                println!("\n\nOptions:");
                println!("  -h, --help\t\t\tPrint this help message.");
                println!(
                    "  -p, --pendulums\t\tNumber of pendulums to simulate. [{}]",
                    amt_pend
                );
                println!("  -n\t\t\t\tAlias for -p.");
                println!("  -v, --vary\t\t\tVary the angle, length1, length2, mass1, or mass2. [angle]");
                println!(
                    "  -s, --separation\t\tSeparation between pendulums. [{}] (used only when varying angle)",
                    amt_sep
                );
                println!(
                    "  -m1, --mass1\t\t\tMass of pendulum part 1. [{}] (used as max mass1 when varying mass1)",
                    settings.max_m1
                );
                println!(
                    "  -m2, --mass2\t\t\tMass of pendulum part 2. [{}] (used as max mass2 when varying mass2)",
                    settings.max_m2
                );
                println!(
                    "  -r1, --radius1\t\t\tLength of pendulum part 1. [{}]",
                    settings.r1
                );
                println!(
                    "  -r2, --radius2\t\t\tLength of pendulum part 2. [{}]",
                    settings.r2
                );
                println!(
                    "  -mag, --magnification\t\tPosition multiplier. [{}]",
                    settings.mag
                );
                println!("  -g, --gravity\t\t\tGravity/Speed[ish]. [{}]",
                         settings.g
                );
                println!(
                    "  -pt, --transparency\t\tTransparency of each pendulum. [{}]",
                    settings.pend_transp
                );
                println!(
                    "  -pw, --width\t\t\tLine width of pendulums. [{}]\n",
                    settings.pend_width
                );
                println!(" -speed\t\t\t\tSpeed of the simulation. [{}]", settings.speed);
                println!("  -q, --quality\t\t\tQuality of the pendulums. (1-3) [1]");
                println!("  -c, --compile\t\t\tCompile the frames into a video, suitable for large amounts of pendulums. [false] ");
                println!("  -f, --frames\t\t\tNumber of frames to compile. [50]");

                return;
            }
            "-p" | "--pendulums" | "-n" => {
                amt_pend = args[i + 1].parse().unwrap();
            }
            "-v" | "--vary" => {
                settings.varying = match args[i + 1].as_str() {
                    "angle" => VaryingType::Angle,
                    "length1" => VaryingType::Length1,
                    "length2" => VaryingType::Length2,
                    "mass1" => VaryingType::Mass1,
                    "mass2" => VaryingType::Mass2,
                    _ => VaryingType::Angle,
                };
            }
            "-s" | "--separation" => {
                amt_sep = args[i + 1].parse().unwrap();
            }
            "-m1" | "--mass1" => {
                settings.max_m1 = args[i + 1].parse().unwrap();
            }
            "-m2" | "--mass2" => {
                settings.max_m2 = args[i + 1].parse().unwrap();
            }
            "-r1" | "--radius1" => {
                settings.r1 = args[i + 1].parse().unwrap();
            }
            "-r2" | "--radius2" => {
                settings.r2 = args[i + 1].parse().unwrap();
            }
            "-mag" | "--magnification" => {
                settings.mag = args[i + 1].parse().unwrap();
            }
            "-g" | "--gravity" => {
                settings.g = args[i + 1].parse().unwrap();
            }
            "-pt" | "--transparency" => {
                settings.pend_transp = args[i + 1].parse().unwrap();
            }
            "-pw" | "--width" => {
                settings.pend_width = args[i + 1].parse().unwrap();
            }
            "-speed" => {
                settings.speed = args[i + 1].parse().unwrap();
            }
            "-q" | "--quality" => {
                settings.quality = match args[i + 1].parse().unwrap() {
                    1 => Quality::Low,
                    2 => Quality::Medium,
                    3 => Quality::High,
                    _ => Quality::Low,
                };
            }
            "-c" | "--compile" => {
                compile = true;
            }
            "-f" | "--frames" => {
                compile_frames = args[i + 1].parse().unwrap();
            }
            _ => {
                // If the argument is not found print a message (will log on numbers too tho).
                if args[i].contains("-") {
                    println!("Unknown argument: {}", args[i]);
                }
            }
        }
    }

    // If the compile flag is set, compile the frames and exit.




    // Innit the pendulums
    let mut pends = Vec::new();
    for i in 0..amt_pend {
        match settings.varying {
            VaryingType::Angle => {
                pends.push(Pendulum::new(
                    -2.0 + (i as f64 * (amt_sep / amt_pend as f64)),
                    -2.0 + (i as f64 * (amt_sep / amt_pend as f64)),
                    settings.r1,
                    settings.r2,
                    settings.max_m1,
                    settings.max_m2,
                    Hsl::from(
                        i as f32 * (360.0 / amt_pend as f32),
                        100.0f32,
                        50.0f32,
                    )
                        .to_rgb(),
                ));
            }
            VaryingType::Length1 => {
                pends.push(Pendulum::new(
                    -2.0,
                    -2.0,
                    settings.r1 + (i as f64 * (settings.max_m1 / amt_pend as f64)),
                    settings.r2,
                    settings.max_m1,
                    settings.max_m2,
                    Hsl::from(
                        i as f32 * (360.0 / amt_pend as f32),
                        100.0f32,
                        50.0f32,
                    )
                        .to_rgb(),
                ));
            }
            VaryingType::Length2 => {
                pends.push(Pendulum::new(
                    -2.0,
                    -2.0,
                    settings.r1,
                    settings.r2 + (i as f64 * (settings.max_m2 / amt_pend as f64)),
                    settings.max_m1,
                    settings.max_m2,
                    Hsl::from(
                        i as f32 * (360.0 / amt_pend as f32),
                        100.0f32,
                        50.0f32,
                    )
                        .to_rgb(),
                ));
            }
            VaryingType::Mass1 => {
                pends.push(Pendulum::new(
                    -2.0,
                    -2.0,
                    settings.r1,
                    settings.r2,
                    i as f64 * (settings.max_m1 / amt_pend as f64),
                    settings.max_m2,
                    Hsl::from(
                        i as f32 * (360.0 / amt_pend as f32),
                        100.0f32,
                        50.0f32,
                    )
                        .to_rgb(),
                ));
            }
            VaryingType::Mass2 => {
                pends.push(Pendulum::new(
                    -2.0,
                    -2.0,
                    settings.r1,
                    settings.r2,
                    settings.max_m1,
                    i as f64 * (settings.max_m2 / amt_pend as f64),
                    Hsl::from(
                        i as f32 * (360.0 / amt_pend as f32),
                        100.0f32,
                        50.0f32,
                    )
                        .to_rgb(),
                ));
            }
        }
    }

    if compile {
        compile_pngs::main(compile_frames, pends, amt_sep, settings);
        return;
    }
    
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new(
        format!(
            "Double Pendulum Simulator! [{} pends, {} deg]",
            amt_pend, amt_sep
        ),
        [600, 600],
    )
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create and run the app
    let mut app = App {
        gl: GlGraphics::new(opengl),
        pends,
        settings,
        fps_counter: 0,
        last_update: Instant::now(),
    };

    // Event Loop
    let mut event_settings = EventSettings::new();
    event_settings.ups = 60;
    event_settings.max_fps = 60;
    let mut events = Events::new(event_settings);
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}
// mod main_processing;
//
// fn main() {
//     main_processing::main();
// }

