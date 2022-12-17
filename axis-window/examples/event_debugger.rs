/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

extern crate axis_window as window;

fn main() {
    // Initialize all of our window system objects.
    let context: window::Context<()> =
        window::Context::open_default().expect("can't open window system context");
    let device = context.default_device();
    let window = device.new_window()
                       .visible()
                       .with_title("Event Debugger")
                       .build(())
                       .expect("can't create main window");

    // The main loop object contains a flag that determines when the main loop will terminate.
    let main_loop = window::MainLoop::new(window::UpdateKind::Passive);

    // Run our main loop.
    context.run(&main_loop, |event| {
        // Report the event.
        println!("{:?}", event);

        // Handle the events that are significant to this program.
        match event {
            window::Event::Close { .. } => window.destroy(),
            window::Event::Destroy { .. } => main_loop.quit(),
            _ => (),
        }
    }).expect("main loop failed");
}
