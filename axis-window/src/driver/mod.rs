/*
 * Copyright (c) 2023 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/// Win32 driver implementation.
#[cfg(all(feature = "win32-driver", target_os = "windows"))]
pub mod win32;

/// X11 driver implementation.
#[cfg(all(feature = "x11-driver", any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "linux",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub mod x11;
