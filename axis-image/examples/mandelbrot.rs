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
use math::{Rect, Vector2};

const BOUNDS: Rect<f64> = Rect::new(-2.0, -1.0, 1.0, 1.0);
const COLOR_EXPONENT: f64 = 0.15;
const IMAGE_SIZE: Vector2<usize> = Vector2::new(601, 401);
const MAX_COLOR: Rgb<f64> = Rgb::new(0.2, 0.7, 1.0);
const MAX_ITERS: u32 = 1000;

fn transform(pos: Vector2<usize>) -> Vector2<f64> {
    Vector2 {
        x: BOUNDS.0.x + pos.x as f64 / (IMAGE_SIZE.x - 1) as f64 * BOUNDS.width(),
        y: BOUNDS.0.y + pos.y as f64 / (IMAGE_SIZE.y - 1) as f64 * BOUNDS.height(),
    }
}

fn mandelbrot(pos: Vector2<f64>) -> Option<u32> {
    let cr = pos.x;
    let ci = pos.y;
    let mut zr = cr;
    let mut zi = ci;

    for n in 0..(MAX_ITERS + 1) {
        if zr.abs() > 2.0 {
            return Some(n);
        }
        let t = zr * zr - zi * zi + cr;
        zi = 2.0 * zr * zi + ci;
        zr = t;
    }

    None
}

fn colorize(n: Option<u32>) -> Rgb<u8> {
    let f = match n {
        None => return color::BLACK,
        Some(n) => f64::powf(n as f64 / MAX_ITERS as f64, COLOR_EXPONENT),
    };
    let color = Rgb::new(MAX_COLOR.r * f, MAX_COLOR.g * f, MAX_COLOR.b * f);
    color.into_color_lossy()
}

fn main() {
    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR")
                                     .expect("missing CARGO_MANIFEST_DIR"));
    let out_dir = manifest_dir.join("output");
    std::fs::create_dir_all(&out_dir).expect("can't create output directory");
    let out_path = out_dir.join("mandelbrot.png");
    let _ = writeln!(std::io::stderr(), "encoding mandelbrot to '{}'...", out_path.display());
    let image = image::generate(IMAGE_SIZE, |pos| colorize(mandelbrot(transform(pos))));
    image.encode_png().write_file(&out_path).expect("encode/write failed");
}
