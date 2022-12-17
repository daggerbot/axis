/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

extern crate axis_color as color;
extern crate axis_image as image;
extern crate axis_math as math;

use std::io::Write;
use std::path::PathBuf;

use color::{IntoColorLossy, Rgb};
use image::Image;
use math::{Complex, Rect, Vector2};

const BOUNDS: Rect<f64> = Rect::new(-2.0, -1.0, 1.0, 1.0);
const COLOR_EXPONENT: f64 = 0.15;
const IMAGE_SIZE: Vector2<usize> = Vector2::new(601, 401);
const MAX_COLOR: Rgb<f64> = Rgb::new(0.2, 0.7, 1.0);
const MAX_ITERS: u32 = 1000;

fn transform(pos: Vector2<usize>) -> Vector2<f64> {
    BOUNDS.0 + pos.to_lossy::<f64>() / (IMAGE_SIZE - (1, 1)).to_lossy::<f64>() * BOUNDS.size()
}

fn mandelbrot(pos: Vector2<f64>) -> Option<u32> {
    let c = Complex::from(pos);
    let mut z = c;
    for n in 0..(MAX_ITERS + 1) {
        if z.0.abs() > 2.0 {
            return Some(n);
        }
        z = z * z + c;
    }
    None
}

fn colorize(n: Option<u32>) -> Rgb<u8> {
    let f = match n {
        None => return color::BLACK,
        Some(n) => f64::powf(n as f64 / MAX_ITERS as f64, COLOR_EXPONENT),
    };
    (MAX_COLOR * f).into_color_lossy()
}

fn main() {
    let mut stderr = std::io::stderr();
    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR")
                       .expect("missing CARGO_MANIFEST_DIR"));
    let out_dir = manifest_dir.join("output");
    let out_path = out_dir.join("mandelbrot.png");

    let _ = writeln!(stderr, "generating mandelbrot...");
    let image = image::generate(IMAGE_SIZE,
                                |pos| colorize(mandelbrot(transform(pos)))).to_vec_image();

    let _ = writeln!(stderr, "saving image to '{}'...", out_path.display());
    std::fs::create_dir_all(&out_dir).expect("can't create output directory");
    image.encode_png()
         .write_file(&out_path)
         .expect("write failed");
}
