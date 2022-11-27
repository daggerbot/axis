/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/// Win32 window system driver.
#[cfg(all(feature = "win32-driver", target_os = "windows"))]
pub mod win32;
/// X11 window system driver.
#[cfg(x11_enabled)]
pub mod x11;
