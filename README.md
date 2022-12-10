# axis - collection of math and multimedia libraries for Rust

[![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-brightgreen.svg)](https://opensource.org/licenses/MPL-2.0)

Axis can be described as something like SDL (Simple DirectMedia Layer) written in pure Rust and for use in Rust.
There are plenty of other Rust crates around that accomplish the same tasks, but I always seem to have issues with some of the design decisions in these libraries.
Axis crates are also developed alongside each other in a single repository and are designed to work very well together.
They tend to have few, if any, external dependencies and try to gate them behind optional cargo features when possible.

Axis is in very early development stages, but I'll try to show my progress in this readme.

## axis-color

Color types and conversions.

Currently implemented at the time of writing:
* Types: `Alpha`, `Lum`, `LumAlpha`, `Red`, `Rg`, `Rgb`, `Rgba`
* Color traits: `Color`, `FromColor`, `FromColorLossy`, `IntoColor`, `IntoColorLossy`, `WithAlpha`
* Color component traits: `Component`, `FromComponent`, `FromComponentLossy`, `IntoComponent`, `IntoComponentLossy`

## axis-image

Types, traits, and codecs for raster images.

Currently implemented at the time of writing:
* Image types: `Bitmap`, `Generate`, `Subimage`, `SubimageMut`, `VecImage`
* Image mapping types: `Cloned`, `Convert`, `ConvertLossy`, `Copied`, `Map`, `To`, `WithMask`, `Zip`
* Traits: `Image`, `ImageMut`
* Codecs: PNG (still has some open bugs)

The [`mandelbrot`](axis-image/examples/mandelbrot.rs) example generates the following image:

![mandelbrot](media/mandelbrot.png)

```
cargo run --example mandelbrot --features png
```

## axis-math

General purpose math types and traits with some emphasis on computer graphics.
Most of the same functionality can be found in the `cgmath` and `num-traits` crates, but I had issues with how some parts of these crates were designed.

Currently implemented at the time of writing:
* Types: `Complex`, `Rect`, `Vector2`, `Vector3`, `Vector4`
* Number traits: `Continuous`, `Identity`, `IntLimits`, `Scalar`, `Zero`
* Conversion traits: `FromComposite`, `FromCompositeLossy`, `FromLossy`, `IntoComposite`, `IntoCompositeLossy`, `IntoLossy`, `TryFromComposite`, `TryIntoComposite`
* Operator traits: `DivCeil`, `DivRem`
* Checked operator traits: `TryAdd`, `TryDiv`, `TryMul`, `TryNeg`, `TrySub`
* Wrapping operator traits: `WrappingAdd`, `WrappingMul`, `WrappingSub`
* Other traits: `Saturate`

## axis-window

Window system library.
Handles creation, manipulation, and input of windows.
Each supported platform/driver is implemented through traits with an `I` prefix.
Boxed types supporting any driver (including those implemented outside of the crate) are provided for simplified use cases where the code should run the same on any platform.
Rendering will be provided through additional planned crates (`axis-draw` and `axis-gl`).

Currently implemented at the time of writing:
* Main boxed types: `Context`, `Device`, `PixelFormat`, `Window`, `WindowBuilder`
* Corresponding traits: `IContext`, `IDevice`, `IPixelFormat`, `IWindow`, `IWindowBuilder`
* Supported platforms: Win32, Unix/X11
* Planned platforms: macOS, Unix/Wayland

The `event_debugger` example can be used to log input events to get an idea of how to handle them.
```
cargo run --example event_debugger
```
