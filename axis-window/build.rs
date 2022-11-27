/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

fn main() {
    match std::env::var("CARGO_CFG_TARGET_OS").expect("missing CARGO_CFG_TARGET_OS").as_str() {
        "dragonfly" | "freebsd" | "netbsd" |"openbsd" => {
            if std::env::var_os("CARGO_FEATURE_X11_DRIVER").is_some() {
                println!("cargo:rustc-cfg=x11_enabled");
            }
        },
        "linux" => {
            if std::env::var_os("CARGO_FEATURE_X11_DRIVER").is_some() {
                println!("cargo:rustc-cfg=x11_enabled");
            }
        },
        _ => (),
    }
}
