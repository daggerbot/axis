/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#[allow(unused_imports)]
#[cfg(feature = "lazy_static")]
#[macro_use]
extern crate lazy_static;
#[cfg(feature = "libc")]
extern crate libc;
#[allow(unused_imports)]
#[cfg(feature = "log")]
#[macro_use]
extern crate log;
extern crate axis_math as math;
#[cfg(all(feature = "winapi", target_os = "windows"))]
extern crate winapi;
#[cfg(all(feature = "x11-sys", x11_enabled))]
extern crate x11_sys;
#[cfg(x11_enabled)]
extern crate xcb_sys;

#[allow(unused_macros)]
#[macro_use]
mod macros;

/// Contains code for window system drivers enable with their respective cargo features.
pub mod driver;

mod context;
mod device;
mod error;
mod event;
mod pixel_format;
#[allow(dead_code)]
mod util;
mod window;

pub use context::{Context, IContext, MainLoop};
pub use device::{Device, Devices, IDevice};
pub use error::{Error, ErrorKind, Result};
pub use event::{Event, UpdateKind};
pub use pixel_format::{IPixelFormat, PixelFormat, PixelFormats};
pub use window::{IWindow, IWindowBuilder, Window, WindowBuilder, WindowKind, WindowPos};

/// Pixel coordinate type.
pub type Coord = i32;
