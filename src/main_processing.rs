// extern crate glutin_window;
// extern crate graphics;
// extern crate opengl_graphics;
// extern crate piston;

extern crate processing as p5;

use colors_transform::{Color, Hsl, Rgb};

// use glutin_window::GlutinWindow as Window;
// use graphics::types::Matrix2d;
// use graphics::{DrawState, Line};
// use opengl_graphics::{GlGraphics, OpenGL};
// use piston::event_loop::{EventSettings, Events};
// use piston::input::{RenderArgs, RenderEvent};
// use piston::window::WindowSettings;
// use piston::{UpdateArgs, UpdateEvent};

use std::env;
use std::f64::consts::PI;
use std::time::Instant;

use p5::shapes::line::Line;

#[derive(Clone, Copy)]
struct Pendulum {
    r1: f64,
    r2: f64,
    a1: f64,
    a2: f64,
    a1_v: f64,
    a2_v: f64,
    color: Rgb,
}

#[derive(Clone, Copy)]
struct Settings {
    g: f64,
    m1: f64,
    m2: f64,
    // sep: f64,
    mag: f64,
    pend_transp: f64,
    pend_width: f64,
    // amt_pend: i32,
}

impl Settings {
    fn new() -> Settings {
        Settings {
            g: 0.1,
            m1: 10.0,
            m2: 10.0,
            // sep: 0.1,
            mag: 2.0,
            pend_transp: 0.05,
            pend_width: 1.5,
            // amt_pend: 50_000,
        }
    }
}

impl Pendulum {
    // Constructor
    fn new(d: f64, d2: f64, color: Rgb) -> Pendulum {
        Pendulum {
            r1: 125.0,
            r2: 125.0,
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
        screen: &mut p5::Screen,
        screen_mid: [u32; 2],
    ) -> Pendulum {
        let a1 = self.a1;
        let a2 = self.a2;
        let a1_v = self.a1_v;
        let a2_v = self.a2_v;
        let r1 = self.r1;
        let r2 = self.r2;
        let m1 = settings.m1;
        let m2 = settings.m2;
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

        // let line_t = Line::new(
        //     [
        //         self.color.get_red() as f32 / 255.0,
        //         self.color.get_blue() as f32 / 255.0,
        //         self.color.get_green() as f32 / 255.0,
        //         settings.pend_transp as f32,
        //     ],
        //     settings.pend_width,
        // );

        let mag = settings.mag;

        // Most P5 Functions take &[f32] as arguments instead of just f32 for some reason
        screen.stroke(
            &[self.color.get_red() as f32 / 255.0],
            &[
                self.color.get_blue() as f32 / 255.0,
                self.color.get_green() as f32 / 255.0,
            ],
            &[settings.pend_transp as f32],
            &[settings.pend_transp as f32],
        );
        screen.stroke_weight(settings.pend_width as f32);

        // line is new(&screen x1, y1, z1, x2, y2, z2)
        let line = Line::new(
            &screen,
            &[0.0 + screen_mid[0] as f64],
            &[0.0 + screen_mid[1] as f64],
            &[0.0],
            &[x1 * mag + screen_mid[0] as f64],
            &[y1 * mag + screen_mid[1] as f64],
            &[0.0],
        ).unwrap();
        screen.draw(&line).expect("Failed to draw line");

        let line = Line::new(
            &screen,
            &[x1 * mag + screen_mid[0] as f64],
            &[y1 * mag + screen_mid[1] as f64],
            &[0.0],
            &[x2 * mag + screen_mid[0] as f64],
            &[y2 * mag + screen_mid[1] as f64],
            &[0.0],
        ).unwrap();
        screen.draw(&line).expect("Failed to draw line");

        self.a1_v += a1_a;
        self.a2_v += a2_a;
        self.a1 += a1_v;
        self.a2 += a2_v;

        self
    }
}

pub struct App<'a> {
    screen: p5::Screen<'a>,
    pends: Vec<Pendulum>,
    settings: Settings,
    fps_counter: u32,
    last_update: Instant,
}

impl App<'_> {
    fn render(&mut self) {
        use graphics::*;

        let (mid_x, mid_y) = (self.screen.width() / 2, self.screen.height() / 2);

        // Clear the screen.
        self.screen.fill(&[0.2], &[0.2], &[0.2], &[1.0]);

        for ui in 0..self.pends.len() {
            //((time/0.5) as u8)  {

            self.pends[ui] =
                self.pends[ui].update_draw(self.settings, &mut self.screen, [mid_x, mid_y]);
        }
    }

    // Update function to print the fps to the console.
    fn update(&mut self) {
        self.screen.poll_events();
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
                println!(
                    "  -s, --separation\t\tSeparation between pendulums. [{}]",
                    amt_sep
                );
                println!(
                    "  -m1, --mass1\t\t\tMass of pendulum part 1. [{}]",
                    settings.m1
                );
                println!(
                    "  -m2, --mass2\t\t\tMass of pendulum part 2. [{}]",
                    settings.m2
                );
                println!(
                    "  -mag, --magnification\t\tPosition multiplier. [{}]",
                    settings.mag
                );
                println!("  -g, --gravity\t\t\tGravity/Speed[ish]. [{}]", settings.g);
                println!(
                    "  -pt, --transparency\t\tTransparency of each pendulum. [{}]",
                    settings.pend_transp
                );
                println!(
                    "  -pw, --width\t\t\tLine width of pendulums. [{}]\n",
                    settings.pend_width
                );

                return;
            }
            "-p" | "--pendulums" => {
                amt_pend = args[i + 1].parse().unwrap();
            }
            "-s" | "--separation" => {
                amt_sep = args[i + 1].parse().unwrap();
            }
            "-m1" | "--mass1" => {
                settings.m1 = args[i + 1].parse().unwrap();
            }
            "-m2" | "--mass2" => {
                settings.m2 = args[i + 1].parse().unwrap();
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
            _ => {
                // If the argument is not found print a message (will log on numbers too tho).
                if args[i].contains("-") {
                    println!("Unknown argument: {}", args[i]);
                }
            }
        }
    }

    // let opengl = OpenGL::V3_2;
    //
    // // Create a Glutin window.
    // let mut window: Window = WindowSettings::new(
    //     format!(
    //         "Double Pendulum Simulator! [{} pends, {} deg]",
    //         amt_pend, amt_sep
    //     ),
    //     [600, 600],
    // )
    //     .graphics_api(opengl)
    //     .exit_on_esc(true)
    //     .build()
    //     .unwrap();

    let mut screen = p5::Screen::new(600, 600, false, false, false).unwrap();

    // Innit the pendulums
    let mut pends = Vec::new();
    for i in 0..amt_pend {
        pends.push(Pendulum::new(
            -2.0 + (i as f64 * (amt_sep / amt_pend as f64)),
            -2.0 + (i as f64 * (amt_sep / amt_pend as f64)),
            Hsl::from(
                i as f32 * (360.0 / amt_pend as f32),
                100.0 as f32,
                50.0 as f32,
            )
            .to_rgb(),
        ));
    }

    // Create and run the app
    let mut app = App {
        screen,
        pends,
        settings,
        fps_counter: 0,
        last_update: Instant::now(),
    };

    app.screen.space_wait();
    app.render();
    app.update();
    // while true {
    //     app.update();
    //     app.render();
    // }

    // Event Loop
    // let mut event_settings = EventSettings::new();
    // event_settings.ups = 60;
    // event_settings.max_fps = 60;
    // let mut events = Events::new(event_settings);
    // while let Some(e) = events.next(&mut window) {
    //     if let Some(args) = e.render_args() {
    //         app.render(&args);
    //     }
    //
    //     if let Some(args) = e.update_args() {
    //         app.update(&args);
    //     }
    // }
}
