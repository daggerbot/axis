/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::error::Error;
use std::fmt::{Display, Formatter};

use math::{Saturate, TrySub, Vector2};

use crate::util::div_ceil;

/// Enumeration of PNG interlace methods.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum InterlaceMethod {
    Adam7 = 1,
}

impl InterlaceMethod {
    /// Returns the byte value corresponding to the interlace method.
    pub fn as_byte(interlace_method: Option<InterlaceMethod>) -> u8 {
        match interlace_method {
            None => 0,
            Some(interlace_method) => interlace_method as u8,
        }
    }

    /// Gets the interlace method with the specified byte value.
    pub fn from_byte(byte: u8) -> Result<Option<InterlaceMethod>, InvalidInterlaceMethod> {
        match byte {
            0 => Ok(None),
            1 => Ok(Some(InterlaceMethod::Adam7)),
            _ => Err(InvalidInterlaceMethod(byte)),
        }
    }
}

impl InterlaceMethod {
    const fn description(self) -> &'static str {
        match self {
            InterlaceMethod::Adam7 => "adam7",
        }
    }
}

impl Display for InterlaceMethod {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        fmt.write_str(self.description())
    }
}

/// Raised when an invalid PNG interlace method is encountered.
#[derive(Clone, Copy, Debug)]
pub struct InvalidInterlaceMethod(pub u8);

impl InvalidInterlaceMethod {
    const DESCRIPTION: &'static str = "invalid png interlace method";
}

impl Display for InvalidInterlaceMethod {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}: {}", Self::DESCRIPTION, self.0)
    }
}

impl Error for InvalidInterlaceMethod {
    fn description(&self) -> &str {
        Self::DESCRIPTION
    }
}

/// Type returned by iterating an `Interlacer`.
#[derive(Clone, Copy)]
pub enum InterlacerItem {
    BeginPass { size: Vector2<usize> },
    Pixel { pos: Vector2<usize> },
}

/// Iterates pixel positions without interlacing.
pub struct DefaultInterlacer {
    image_size: Vector2<usize>,
    pos: Vector2<usize>,
    queue: Option<InterlacerItem>,
}

impl DefaultInterlacer {
    pub fn new(image_size: Vector2<usize>) -> DefaultInterlacer {
        DefaultInterlacer {
            image_size,
            pos: Vector2::new(0, 0),
            queue: Some(InterlacerItem::BeginPass { size: image_size }),
        }
    }
}

impl Iterator for DefaultInterlacer {
    type Item = InterlacerItem;

    fn next(&mut self) -> Option<InterlacerItem> {
        if let Some(item) = self.queue.take() {
            return Some(item);
        } else if self.pos.y == self.image_size.y {
            return None;
        }

        let item = InterlacerItem::Pixel { pos: self.pos };
        self.pos.x += 1;
        if self.pos.x == self.image_size.x {
            self.pos.x = 0;
            self.pos.y += 1;
        }
        Some(item)
    }
}

/// Iterates pixel positions according to the Adam7 interlace method.
pub struct Adam7Interlacer {
    image_size: Vector2<usize>,
    offset_x: usize,
    pass: u8,
    pos: Vector2<usize>,
    queue: Option<InterlacerItem>,
    stride: Vector2<usize>,
}

impl Adam7Interlacer {
    pub fn new(image_size: Vector2<usize>) -> Adam7Interlacer {
        Adam7Interlacer {
            image_size,
            offset_x: 0,
            pass: 0,
            pos: Vector2::new(0, 0),
            queue: Some(InterlacerItem::BeginPass {
                size: Vector2 {
                    x: div_ceil(image_size.x, 8),
                    y: div_ceil(image_size.y, 8),
                },
            }),
            stride: Vector2::new(8, 8),
        }
    }
}

impl Iterator for Adam7Interlacer {
    type Item = InterlacerItem;

    fn next(&mut self) -> Option<InterlacerItem> {
        if let Some(item) = self.queue.take() {
            return Some(item);
        } else if self.pass == 7 {
            return None;
        }

        loop {
            if self.pos.y >= self.image_size.y {
                self.pass += 1;
                match self.pass {
                    1 => {
                        self.pos = Vector2::new(4, 0);
                        self.stride = Vector2::new(8, 8);
                    },
                    2 => {
                        self.pos = Vector2::new(0, 4);
                        self.stride = Vector2::new(4, 8);
                    },
                    3 => {
                        self.pos = Vector2::new(2, 0);
                        self.stride = Vector2::new(4, 4);
                    },
                    4 => {
                        self.pos = Vector2::new(0, 2);
                        self.stride = Vector2::new(2, 4);
                    },
                    5 => {
                        self.pos = Vector2::new(1, 0);
                        self.stride = Vector2::new(2, 2);
                    },
                    6 => {
                        self.pos = Vector2::new(0, 1);
                        self.stride = Vector2::new(1, 2);
                    },
                    7 => return None,
                    _ => unreachable!(),
                }
                self.offset_x = self.pos.x;
                return Some(InterlacerItem::BeginPass {
                    size: Vector2 {
                        x: div_ceil(
                            self.image_size.x.try_sub(self.pos.x).saturate(),
                            self.stride.x,
                        ),
                        y: div_ceil(
                            self.image_size.y.try_sub(self.pos.y).saturate(),
                            self.stride.y,
                        ),
                    },
                });
            } else if self.pos.x >= self.image_size.x {
                self.pos.x = self.offset_x;
                self.pos.y += self.stride.y;
            } else {
                let item = InterlacerItem::Pixel { pos: self.pos };
                self.pos.x += self.stride.x;
                return Some(item);
            }
        }
    }
}

/// Iterates pixel positions according to an interlace method.
pub enum Interlacer {
    Adam7(Adam7Interlacer),
    Default(DefaultInterlacer),
}

impl Interlacer {
    pub fn new(image_size: Vector2<usize>, method: Option<InterlaceMethod>) -> Interlacer {
        match method {
            None => Interlacer::Default(DefaultInterlacer::new(image_size)),
            Some(InterlaceMethod::Adam7) => Interlacer::Adam7(Adam7Interlacer::new(image_size)),
        }
    }
}

impl Iterator for Interlacer {
    type Item = InterlacerItem;

    fn next(&mut self) -> Option<InterlacerItem> {
        match *self {
            Interlacer::Default(ref mut inner) => inner.next(),
            Interlacer::Adam7(ref mut inner) => inner.next(),
        }
    }
}
