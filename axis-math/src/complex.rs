/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::ops::{Add, Div, Mul, Sub};

use crate::vector::Vector2;

/// Complex number type.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Complex<T>(pub T, pub T);

impl<T> Add<T> for Complex<T>
where
    T: Add<Output = T>,
{
    type Output = Complex<T>;

    fn add(self, rhs: T) -> Self::Output {
        Complex(self.0 + rhs, self.1)
    }
}

impl<T> Add for Complex<T>
where
    T: Add,
{
    type Output = Complex<<T as Add>::Output>;

    fn add(self, rhs: Complex<T>) -> Self::Output {
        Complex(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<T> Div<T> for Complex<T>
where
    T: Copy + Div,
{
    type Output = Complex<<T as Div>::Output>;

    fn div(self, rhs: T) -> Self::Output {
        Complex(self.0 / rhs, self.1 / rhs)
    }
}

impl<T> Div for Complex<T>
where
    T: Add<Output = T> + Copy + Div<Output = T> + Mul<Output = T> + Sub<Output = T>,
{
    type Output = Complex<T>;

    fn div(self, rhs: Complex<T>) -> Self::Output {
        let d = rhs.0 * rhs.0 + rhs.1 * rhs.1;
        Complex((self.0 * rhs.0 + self.1 * rhs.1) / d,
                (self.1 * rhs.0 - self.0 * rhs.1) / d)
    }
}

impl<T> From<Vector2<T>> for Complex<T> {
    fn from(v: Vector2<T>) -> Complex<T> { Complex(v.x, v.y) }
}

impl<T> Mul<T> for Complex<T>
where
    T: Copy + Mul,
{
    type Output = Complex<<T as Mul>::Output>;

    fn mul(self, rhs: T) -> Self::Output {
        Complex(self.0 * rhs, self.1 * rhs)
    }
}

impl<T> Mul for Complex<T>
where
    T: Add<Output = T> + Copy + Mul<Output = T> + Sub<Output = T>,
{
    type Output = Complex<T>;

    fn mul(self, rhs: Complex<T>) -> Self::Output {
        Complex(self.0 * rhs.0 - self.1 * rhs.1, self.0 * rhs.1 + self.1 * rhs.0)
    }
}

impl<T> Sub<T> for Complex<T>
where
    T: Sub<Output = T>,
{
    type Output = Complex<T>;

    fn sub(self, rhs: T) -> Self::Output {
        Complex(self.0 - rhs, self.1)
    }
}

impl<T> Sub for Complex<T>
where
    T: Sub,
{
    type Output = Complex<<T as Sub>::Output>;

    fn sub(self, rhs: Complex<T>) -> Self::Output {
        Complex(self.0 - rhs.0, self.1 - rhs.1)
    }
}
