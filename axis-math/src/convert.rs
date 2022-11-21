/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/// Attempts to convert composite numeric types.
///
/// This is used instead of `TryFrom` to avoid conflicts with blanket implementations.
pub trait TryFromComposite<F>: Sized {
    type Error: Sized;
    fn try_from_composite(from: F) -> Result<Self, Self::Error>;
}

/// Attempts to convert composite numeric types.
///
/// This is used instead of `TryInto` to avoid conflicts with blanket implementations.
pub trait TryIntoComposite<T>: Sized {
    type Error: Sized;
    fn try_into_composite(self) -> Result<T, Self::Error>;
}

impl<F, T: TryFromComposite<F>> TryIntoComposite<T> for F {
    type Error = T::Error;

    fn try_into_composite(self) -> Result<T, Self::Error> {
        T::try_from_composite(self)
    }
}
