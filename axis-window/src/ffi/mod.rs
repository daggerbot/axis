/*
 * Copyright (c) 2023 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#[cfg(all(feature = "libc", unix))]
pub mod posix;

#[cfg(all(feature = "winapi", target_os = "windows"))]
pub mod win32;
