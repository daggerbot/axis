/*
 * Copyright (c) 2023 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/// Constructs an error.
macro_rules! err {
    ($kind:ident) => {
        crate::error::Error::from(crate::error::ErrorKind::$kind)
    };
    ($kind:ident($detail:expr)) => {
        err!($kind).with_detail($detail)
    };
    ($kind:ident[$detail:expr]) => {
        err!($kind).with_detail_string($detail)
    };
    ($kind:ident{$($args:expr),*}) => {
        err!($kind).with_detail_string(format!($($args),*))
    };

    ($kind:ident: $source:expr) => {
        err!($kind).with_source($source)
    };
    ($kind:ident($detail:expr): $source:expr) => {
        err!($kind($detail)).with_source($source)
    };
    ($kind:ident[$detail:expr]: $source:expr) => {
        err!($kind[$detail]).with_source($source)
    };
    ($kind:ident{$($args:expr),*}: $source:expr) => {
        err!($kind{$($args),*}).with_source($source)
    };

    ($kind:ident: ?$source:expr) => {
        err!($kind).maybe_with_source($source)
    };
    ($kind:ident($detail:expr): ?$source:expr) => {
        err!($kind($detail)).maybe_with_source($source)
    };
    ($kind:ident[$detail:expr]: ?$source:expr) => {
        err!($kind[$detail]).maybe_with_source($source)
    };
    ($kind:ident{$($args:expr),*}: ?$source:expr) => {
        err!($kind{$($args),*}).maybe_with_source($source)
    };

    ($kind:ident: ??w) => {
        err!($kind: ?crate::ffi::win32::Error::get())
    };
    ($kind:ident($detail:expr): ??w) => {
        err!($kind($detail): ?crate::ffi::win32::Error::get())
    };
    ($kind:ident[$detail:expr]: ??w) => {
        err!($kind[$detail]: ?crate::ffi::win32::Error::get())
    };
    ($kind:ident{$($args:expr),*}: ??w) => {
        err!($kind{$($args),*}: ?crate::ffi::win32::Error::get())
    };
}

/// No-op.
#[cfg(not(feature = "log"))]
macro_rules! error {
    ($($tt:tt),*) => {};
}
