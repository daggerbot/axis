/*
 * Copyright (c) 2023 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate axis_window as window;

use window::{Event, IClient, IWindow, IWindowBuilder, MainLoop, UpdateMode};

fn main() {
    // Intiailize
    if let Err(err) = simple_logger::init_with_env() {
        panic!("can't initialize logger: {}", err);
    }
    let client = match window::Client::<()>::open_default() {
        Ok(client) => client,
        Err(err) => panic!("can't open window system client: {}", err),
    };
    let window = match client.window().build(()) {
        Ok(window) => window,
        Err(err) => panic!("can't create main window: {}", err),
    };
    if let Err(err) = window.set_visible(true) {
        panic!("can't show window: {}", err);
    }

    // Main loop
    let main_loop = MainLoop::new(UpdateMode::Passive);
    if let Err(err) = client.run(&main_loop, &|event| {
        info!("{:?}", event);
        match event {
            Event::CloseRequest { .. } => window.destroy(),
            Event::Destroy { .. } => main_loop.quit(),
            _ => (),
        }
    }) {
        panic!("can't poll events: {}", err);
    }
}
