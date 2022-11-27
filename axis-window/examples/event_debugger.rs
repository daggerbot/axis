/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

extern crate simple_logger;
extern crate axis_window as window;

fn main() {
    simple_logger::init_with_env().expect("can't initialize logger");
    let context: window::Context<()> = window::Context::open_default()
                                       .expect("can't open window system context");
    let device = context.default_device();
    let _window = device.new_window().build(()).expect("can't create main window");
}
