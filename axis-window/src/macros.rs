/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/// Constructs an [`Error`].
macro_rules! err {
    ($kind:ident) => {{ crate::error::Error::from(crate::error::ErrorKind::$kind) }};
    ($kind:ident($detail:expr)) => {{ err!($kind).with_detail($detail) }};
    ($kind:ident[$detail:expr]) => {{ err!($kind).with_detail_string($detail) }};
    ($kind:ident{$($args:expr),*}) => {{ err!($kind).with_detail_string(format!($($args),*)) }};
    ($kind:ident: $source:expr) => {{ err!($kind).with_source($source) }};
    ($kind:ident($detail:expr): $source:expr) => {{ err!($kind($detail)).with_source($source) }};
    ($kind:ident[$detail:expr]: $source:expr) => {{ err!($kind[$detail]).with_source($source) }};
    ($kind:ident{$($args:expr),*}: $source:expr) => {{
        err!($kind{$($args),*}).with_source($source)
    }};
}

/// No-op error log macro.
#[cfg(not(feature = "log"))]
macro_rules! error {
    ($($args:expr),*) => {};
}
