/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/// Returns the ceiling of an integer quotient.
pub trait DivCeil<Rhs = Self> {
    type Output;
    fn div_ceil(self, rhs: Rhs) -> Self::Output;
}

/// Unifies integer division and remainder as one operation.
pub trait DivRem<Rhs = Self> {
    type Quotient;
    type Remainder;
    fn div_rem(self, rhs: Rhs) -> (Self::Quotient, Self::Remainder);
}

macro_rules! impl_int_ops {
    ($($ty:ty),*) => { $(
        impl DivCeil<$ty> for $ty {
            type Output = $ty;

            fn div_ceil(self, rhs: $ty) -> $ty {
                match self.div_rem(rhs) {
                    (q, 0) => q,
                    #[allow(unused_comparisons)]
                    (q, _) => if (self < 0) == (rhs < 0) { q + 1 } else { q },
                }
            }
        }

        impl DivRem<$ty> for $ty {
            type Quotient = $ty;
            type Remainder = $ty;

            fn div_rem(self, rhs: $ty) -> ($ty, $ty) {
                (self / rhs, self % rhs)
            }
        }
    )* };
}

impl_int_ops!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
