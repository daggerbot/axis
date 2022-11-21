/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::fmt::{Display, Formatter};

const DIV_BY_ZERO_MESSAGE: &'static str = "divided by zero";
const OVERFLOW_MESSAGE: &'static str = "arithmetic overflow";
const UNDERFLOW_MESSAGE: &'static str = "arithmetic underflow";

/// Indicates that a division by zero occurred.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct DivByZeroError;

impl Display for DivByZeroError {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        fmt.write_str(DIV_BY_ZERO_MESSAGE)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DivByZeroError {
    fn description(&self) -> &str { DIV_BY_ZERO_MESSAGE }
}

/// Indicates that an operation resulted in an overflow.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct OverflowError;

impl Display for OverflowError {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        fmt.write_str(OVERFLOW_MESSAGE)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OverflowError {
    fn description(&self) -> &str { OVERFLOW_MESSAGE }
}

/// Indicates that an operation resulted in an underflow.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UnderflowError;

impl Display for UnderflowError {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        fmt.write_str(UNDERFLOW_MESSAGE)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for UnderflowError {
    fn description(&self) -> &str { UNDERFLOW_MESSAGE }
}

/// Indicates that the output value cannot be represented by the output type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RangeError {
    Overflow,
    Underflow,
}

impl Display for RangeError {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        fmt.write_str(match *self {
            RangeError::Overflow => OVERFLOW_MESSAGE,
            RangeError::Underflow => UNDERFLOW_MESSAGE,
        })
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RangeError {
    fn description(&self) -> &str {
        match *self {
            RangeError::Overflow => OVERFLOW_MESSAGE,
            RangeError::Underflow => UNDERFLOW_MESSAGE,
        }
    }
}

impl From<OverflowError> for RangeError {
    fn from(_: OverflowError) -> RangeError { RangeError::Overflow }
}

impl From<UnderflowError> for RangeError {
    fn from(_: UnderflowError) -> RangeError { RangeError::Underflow }
}
