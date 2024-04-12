use std::process::Command;
use colors_transform::{Color, Hsl};
use image::{ImageBuffer, Rgba};
use imageproc::drawing::{Blend, draw_antialiased_line_segment_mut, draw_filled_circle_mut, draw_filled_rect_mut, draw_hollow_circle_mut, draw_line_segment_mut};
use imageproc::rect::Rect;
use crate::{Pendulum, Settings, Quality};

static IMAGE_SIDE_LENGTH: u32 = 1500;


impl Pendulum {
    fn update_draw_img(
        mut self,
        settings: Settings,
        image: &mut Blend<ImageBuffer<Rgba<u8>, Vec<u8>>>,
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


        // let line_t = Line::new(
        //     [
        //         self.color.get_red() as u8,
        //         self.color.get_blue() as u8,
        //         self.color.get_green() as u8,
        //         settings.pend_transp as f32,
        //     ],
        //     settings.pend_width,
        // );

        let mag = settings.mag;

        let color = Rgba([self.color.get_red() as u8, self.color.get_blue() as u8, self.color.get_green() as u8, (settings.pend_transp * 255.0f64) as u8]);

        let midpt = (IMAGE_SIDE_LENGTH / 2) as f64;
        draw_line_segment_mut(image, (midpt as f32, midpt as f32), ((x1 * mag + midpt) as f32, (y1 * mag + midpt) as f32), color);
        draw_line_segment_mut(image, ((x1 * mag + midpt) as f32, (y1 * mag + midpt) as f32), ((x2 * mag + midpt) as f32, (y2 * mag + midpt) as f32), color);


        if settings.quality == Quality::Medium {
            draw_filled_circle_mut(image, ((x1 * mag - (m1 / 2.0) + midpt) as i32, (y1 * mag - (m1 / 2.0) + midpt) as i32), m1 as i32, color);
            draw_filled_circle_mut(image, ((x2 * mag - (m2 / 2.0) + midpt) as i32, (y2 * mag - (m2 / 2.0) + midpt) as i32), m2 as i32, color);
        }
        else if settings.quality == Quality::High {
            let black_transparent = Rgba([0, 0, 0, (settings.pend_transp * 255.0f64) as u8]);
            draw_filled_circle_mut(image, ((x1 * mag + midpt) as i32, (y1 * mag + midpt) as i32), m1 as i32, color);
            draw_filled_circle_mut(image, ((x2 * mag + midpt) as i32, (y2 * mag + midpt) as i32), m2 as i32, color);
            draw_hollow_circle_mut(image, ((x1 * mag + midpt) as i32, (y1 * mag + midpt) as i32), m1 as i32, black_transparent);
            draw_hollow_circle_mut(image, ((x2 * mag + midpt) as i32, (y2 * mag + midpt) as i32), m2 as i32, black_transparent);
        }

        self.a1_v += a1_a * settings.speed;
        self.a2_v += a2_a * settings.speed;
        self.a1 += a1_v * settings.speed;
        self.a2 += a2_v * settings.speed;

        self
    }
}

pub(crate) fn main(compile_frames: i32, mut pends: Vec<Pendulum>, amt_sep: f64, settings: Settings) {
    // Innit the pendulums
    // let mut pends = Vec::new();
    // for i in 0..amt_pend {
    //     pends.push(Pendulum::new(
    //         -2.0 + (i as f64 * (amt_sep / amt_pend as f64)),
    //         -2.0 + (i as f64 * (amt_sep / amt_pend as f64)),
    //         settings.r1,
    //         settings.r2,
    //         Hsl::from(
    //             i as f32 * (360.0 / amt_pend as f32),
    //             100.0f32,
    //             50.0f32,
    //         )
    //             .to_rgb(),
    //     ));
    // }
    let amt_pend = pends.len();
    
    // Try creating the folder /frames if it doesn't exist.
    std::fs::create_dir("frames").unwrap_or_default();

    // Write the frames to a file.
    for frame_i in 0..compile_frames {
        let mut image = Blend(image::RgbaImage::new(IMAGE_SIDE_LENGTH, IMAGE_SIDE_LENGTH));
        // fill the image with (0.2, 0.2, 0.2, 1.0)
        draw_filled_rect_mut(&mut image, Rect::at(0, 0).of_size(1500, 1500), Rgba([51, 51, 51, 255]));
        for i in 0..amt_pend {
            pends[i] = pends[i].update_draw_img(settings, &mut image);
            // if i % 10000 == 0 {println!("Updated pendulum {}", i);}
        }
        image.0.save(format!("frames/{}.png", frame_i)).unwrap();
        println!("Saved frame {}", frame_i);
    }
    println!("Done saving frames!");
    // Stitch the frames together into a video.
    let mut ffmpeg = Command::new("ffmpeg");
    ffmpeg
        .arg("-framerate")
        .arg("10")
        .arg("-i")
        .arg("frames/%d.png")
        .arg("-c:v")
        .arg("libx264")
        .arg("-r")
        .arg("60")
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("output.mp4");
    ffmpeg.output().unwrap();
    // ffmpeg -framerate 10 -i frames/%d.png -c:v libx264 -r 60 -pix_fmt yuv420p output.mp4
    println!("Done stitching frames!");
}