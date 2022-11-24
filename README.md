# axis - collection of math and multimedia libraries for Rust

[![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-brightgreen.svg)](https://opensource.org/licenses/MPL-2.0)

Axis can be thought of as something like SDL (Simple DirectMedia Layer) written in pure Rust for use in Rust.
There are plenty of other Rust crates around that accomplish the same tasks, but I always seem to have issues with some of the design decisions in these libraries.
Axis crates are also developed alongside each other in a single repository and are designed to work very well together.
They tend to have few, if any, external dependencies and try to gate them behind optional cargo features when possible.

Axis is in very early development stages, but I'll post about my progress to various places when there's something interesting to show. So far, all I have is the `mandelbrot` example.

![mandelbrot](media/mandelbrot.png)

The example code that generates this image can be found in [here](axis-image/examples/mandelbrot.rs).
To generate this image, enter
```
cargo run --example mandelbrot --features png
```
