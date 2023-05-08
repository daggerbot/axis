/*
 * Copyright (c) 2023 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

//! Cross-platform window system client library.

extern crate vectorial;

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

#[cfg(all(feature = "win32-driver", target_os = "windows"))]
extern crate winapi;

#[cfg(all(feature = "x11-sys", any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "linux",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
extern crate x11_sys;

#[cfg(all(feature = "x11-driver", any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "linux",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
extern crate xcb_sys;

#[allow(unused_macros)]
#[macro_use]
mod macros;

/// Driver implementations.
pub mod driver;

mod client;
mod error;
mod event;
mod pixel_format;
mod window;

#[allow(dead_code)]
mod ffi;

pub use client::{Client, IClient};
pub use error::{Error, ErrorKind, Result};
pub use event::{Event, MainLoop, UpdateMode};
pub use window::{IWindow, IWindowBuilder, Window, WindowBuilder};

/// Window coordinate type.
pub type Coord = i32;
