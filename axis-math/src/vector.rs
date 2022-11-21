/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::fmt::{Display, Formatter};
use std::ops::{
    Add,
    AddAssign,
    Div,
    DivAssign,
    Mul,
    MulAssign,
    Neg,
    Sub,
    SubAssign,
};

use crate::convert::TryFromComposite;
use crate::num::{Identity, Zero};
use crate::try_ops::{TryAdd, TryDiv, TryMul, TryNeg, TrySub};
use crate::wrapping_ops::{WrappingAdd, WrappingMul, WrappingSub};

/// 2-dimensional vector type.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Vector2<T> {
    pub x: T,
    pub y: T,
}

impl<T: Display> Display for Vector2<T> {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "({}, {})", self.x, self.y)
    }
}

/// 3-dimensional vector type.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Display> Display for Vector3<T> {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "({}, {}, {})", self.x, self.y, self.z)
    }
}

/// 4-dimensional vector type.
///
/// It should be noted that the `w` scalar is not treated differently from the others unless
/// otherwise specified. This is important because 4-dimensional vectors are often used as
/// homogeneous vectors in 3-dimensional clipping space.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Vector4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl<T: Display> Display for Vector4<T> {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}

//--------------------------------------------------------------------------------------------------

macro_rules! impl_unary_ops {
    { $(impl $trait:ident::$fn:ident for $vector:ident($($field:ident),*);)* } => { $(
        impl<T> $trait for $vector<T>
        where
            T: $trait,
        {
            type Output = $vector<<T as $trait>::Output>;

            fn $fn(self) -> Self::Output {
                $vector { $($field: $trait::$fn(self.$field)),* }
            }
        }

        impl<'a, T> $trait for &'a $vector<T>
        where
            &'a T: $trait,
        {
            type Output = $vector<<&'a T as $trait>::Output>;

            fn $fn(self) -> Self::Output {
                $vector { $($field: $trait::$fn(&self.$field)),* }
            }
        }
    )* };
}

macro_rules! impl_scalar_ops {
    { $(impl $trait:ident::$fn:ident for $vector:ident($($field:ident),*);)* } => { $(
        impl<T> $trait<T> for $vector<T>
        where
            T: Copy + $trait,
        {
            type Output = $vector<<T as $trait>::Output>;

            fn $fn(self, rhs: T) -> Self::Output {
                $vector { $($field: $trait::$fn(self.$field, rhs)),* }
            }
        }

        impl<'a, T> $trait<T> for &'a $vector<T>
        where
            T: Copy,
            &'a T: $trait<T>,
        {
            type Output = $vector<<&'a T as $trait<T>>::Output>;

            fn $fn(self, rhs: T) -> Self::Output {
                $vector { $($field: $trait::$fn(&self.$field, rhs)),* }
            }
        }

        impl<'r, T> $trait<&'r T> for $vector<T>
        where
            T: $trait<&'r T>,
        {
            type Output = $vector<<T as $trait<&'r T>>::Output>;

            fn $fn(self, rhs: &'r T) -> Self::Output {
                $vector { $($field: $trait::$fn(self.$field, &rhs)),* }
            }
        }

        impl<'a, 'r, T> $trait<&'r T> for &'a $vector<T>
        where
            &'a T: $trait<&'r T>,
        {
            type Output = $vector<<&'a T as $trait<&'r T>>::Output>;

            fn $fn(self, rhs: &'r T) -> Self::Output {
                $vector { $($field: $trait::$fn(&self.$field, &rhs)),* }
            }
        }
    )* };
}

macro_rules! impl_scalar_assign_ops {
    { $(impl $trait:ident::$fn:ident for $vector:ident($($field:ident),*);)* } => { $(
        impl<T> $trait<T> for $vector<T>
        where
            T: Copy + $trait,
        {
            fn $fn(&mut self, rhs: T) {
                $($trait::$fn(&mut self.$field, rhs);)*
            }
        }

        impl<'r, T> $trait<&'r T> for $vector<T>
        where
            T: $trait<&'r T>,
        {
            fn $fn(&mut self, rhs: &'r T) {
                $($trait::$fn(&mut self.$field, rhs);)*
            }
        }
    )* };
}

macro_rules! impl_vector_ops {
    { $(impl $trait:ident::$fn:ident for $vector:ident($($field:ident),*);)* } => { $(
        impl<T> $trait for $vector<T>
        where
            T: $trait,
        {
            type Output = $vector<<T as $trait>::Output>;

            fn $fn(self, rhs: $vector<T>) -> Self::Output {
                $vector { $($field: $trait::$fn(self.$field, rhs.$field)),* }
            }
        }

        impl<'a, T> $trait<$vector<T>> for &'a $vector<T>
        where
            &'a T: $trait<T>,
        {
            type Output = $vector<<&'a T as $trait<T>>::Output>;

            fn $fn(self, rhs: $vector<T>) -> Self::Output {
                $vector { $($field: $trait::$fn(&self.$field, rhs.$field)),* }
            }
        }

        impl<'r, T> $trait<&'r $vector<T>> for $vector<T>
        where
            T: $trait<&'r T>,
        {
            type Output = $vector<<T as $trait<&'r T>>::Output>;

            fn $fn(self, rhs: &'r $vector<T>) -> Self::Output {
                $vector { $($field: $trait::$fn(self.$field, &rhs.$field)),* }
            }
        }

        impl<'a, 'r, T> $trait<&'r $vector<T>> for &'a $vector<T>
        where
            &'a T: $trait<&'r T>,
        {
            type Output = $vector<<&'a T as $trait<&'r T>>::Output>;

            fn $fn(self, rhs: &'r $vector<T>) -> Self::Output {
                $vector { $($field: $trait::$fn(&self.$field, &rhs.$field)),* }
            }
        }
    )* };
}

macro_rules! impl_vector_assign_ops {
    { $(impl $trait:ident::$fn:ident for $vector:ident($($field:ident),*);)* } => { $(
        impl<T> $trait for $vector<T>
        where
            T: $trait,
        {
            fn $fn(&mut self, rhs: $vector<T>) {
                $($trait::$fn(&mut self.$field, rhs.$field);)*
            }
        }

        impl<'r, T> $trait<&'r $vector<T>> for $vector<T>
        where
            T: $trait<&'r T>,
        {
            fn $fn(&mut self, rhs: &'r $vector<T>) {
                $($trait::$fn(&mut self.$field, &rhs.$field);)*
            }
        }
    )* };
}

macro_rules! impl_unary_try_ops {
    { $(impl $trait:ident::$fn:ident for $vector:ident($($field:ident),*);)* } => { $(
        impl<T> $trait for $vector<T>
        where
            T: $trait,
        {
            type Output = $vector<<T as $trait>::Output>;
            type Error = <T as $trait>::Error;

            fn $fn(self) -> Result<Self::Output, Self::Error> {
                Ok($vector { $($field: $trait::$fn(self.$field)?),* })
            }
        }

        impl<'a, T> $trait for &'a $vector<T>
        where
            &'a T: $trait,
        {
            type Output = $vector<<&'a T as $trait>::Output>;
            type Error = <&'a T as $trait>::Error;

            fn $fn(self) -> Result<Self::Output, Self::Error> {
                Ok($vector { $($field: $trait::$fn(&self.$field)?),* })
            }
        }
    )* };
}

macro_rules! impl_scalar_try_ops {
    { $(impl $trait:ident::$fn:ident for $vector:ident($($field:ident),*);)* } => { $(
        impl<T> $trait<T> for $vector<T>
        where
            T: Copy + $trait,
        {
            type Output = $vector<<T as $trait>::Output>;
            type Error = <T as $trait>::Error;

            fn $fn(self, rhs: T) -> Result<Self::Output, Self::Error> {
                Ok($vector { $($field: $trait::$fn(self.$field, rhs)?),* })
            }
        }

        impl<'a, T> $trait<T> for &'a $vector<T>
        where
            T: Copy,
            &'a T: $trait<T>,
        {
            type Output = $vector<<&'a T as $trait<T>>::Output>;
            type Error = <&'a T as $trait<T>>::Error;

            fn $fn(self, rhs: T) -> Result<Self::Output, Self::Error> {
                Ok($vector { $($field: $trait::$fn(&self.$field, rhs)?),* })
            }
        }

        impl<'r, T> $trait<&'r T> for $vector<T>
        where
            T: $trait<&'r T>,
        {
            type Output = $vector<<T as $trait<&'r T>>::Output>;
            type Error = <T as $trait<&'r T>>::Error;

            fn $fn(self, rhs: &'r T) -> Result<Self::Output, Self::Error> {
                Ok($vector { $($field: $trait::$fn(self.$field, &rhs)?),* })
            }
        }

        impl<'a, 'r, T> $trait<&'r T> for &'a $vector<T>
        where
            &'a T: $trait<&'r T>,
        {
            type Output = $vector<<&'a T as $trait<&'r T>>::Output>;
            type Error = <&'a T as $trait<&'r T>>::Error;

            fn $fn(self, rhs: &'r T) -> Result<Self::Output, Self::Error> {
                Ok($vector { $($field: $trait::$fn(&self.$field, &rhs)?),* })
            }
        }
    )* };
}

macro_rules! impl_vector_try_ops {
    { $(impl $trait:ident::$fn:ident for $vector:ident($($field:ident),*);)* } => { $(
        impl<T> $trait for $vector<T>
        where
            T: $trait,
        {
            type Output = $vector<<T as $trait>::Output>;
            type Error = <T as $trait>::Error;

            fn $fn(self, rhs: $vector<T>) -> Result<Self::Output, Self::Error> {
                Ok($vector { $($field: $trait::$fn(self.$field, rhs.$field)?),* })
            }
        }

        impl<'a, T> $trait<$vector<T>> for &'a $vector<T>
        where
            &'a T: $trait<T>,
        {
            type Output = $vector<<&'a T as $trait<T>>::Output>;
            type Error = <&'a T as $trait<T>>::Error;

            fn $fn(self, rhs: $vector<T>) -> Result<Self::Output, Self::Error> {
                Ok($vector { $($field: $trait::$fn(&self.$field, rhs.$field)?),* })
            }
        }

        impl<'r, T> $trait<&'r $vector<T>> for $vector<T>
        where
            T: $trait<&'r T>,
        {
            type Output = $vector<<T as $trait<&'r T>>::Output>;
            type Error = <T as $trait<&'r T>>::Error;

            fn $fn(self, rhs: &'r $vector<T>) -> Result<Self::Output, Self::Error> {
                Ok($vector { $($field: $trait::$fn(self.$field, &rhs.$field)?),* })
            }
        }

        impl<'a, 'r, T> $trait<&'r $vector<T>> for &'a $vector<T>
        where
            &'a T: $trait<&'r T>,
        {
            type Output = $vector<<&'a T as $trait<&'r T>>::Output>;
            type Error = <&'a T as $trait<&'r T>>::Error;

            fn $fn(self, rhs: &'r $vector<T>) -> Result<Self::Output, Self::Error> {
                Ok($vector { $($field: $trait::$fn(&self.$field, &rhs.$field)?),* })
            }
        }
    )* };
}

macro_rules! impl_scalar_wrapping_ops {
    { $(impl $trait:ident::$fn:ident for $vector:ident($($field:ident),*);)* } => { $(
        impl<T> $trait<T> for $vector<T>
        where
            T: Copy + $trait,
        {
            type Output = $vector<<T as $trait>::Output>;

            fn $fn(self, rhs: T) -> Self::Output {
                $vector { $($field: $trait::$fn(self.$field, rhs)),* }
            }
        }

        impl<'a, T> $trait<T> for &'a $vector<T>
        where
            T: Copy,
            &'a T: $trait<T>,
        {
            type Output = $vector<<&'a T as $trait<T>>::Output>;

            fn $fn(self, rhs: T) -> Self::Output {
                $vector { $($field: $trait::$fn(&self.$field, rhs)),* }
            }
        }

        impl<'r, T> $trait<&'r T> for $vector<T>
        where
            T: $trait<&'r T>,
        {
            type Output = $vector<<T as $trait<&'r T>>::Output>;

            fn $fn(self, rhs: &'r T) -> Self::Output {
                $vector { $($field: $trait::$fn(self.$field, &rhs)),* }
            }
        }

        impl<'a, 'r, T> $trait<&'r T> for &'a $vector<T>
        where
            &'a T: $trait<&'r T>,
        {
            type Output = $vector<<&'a T as $trait<&'r T>>::Output>;

            fn $fn(self, rhs: &'r T) -> Self::Output {
                $vector { $($field: $trait::$fn(&self.$field, &rhs)),* }
            }
        }
    )* };
}

macro_rules! impl_vector_wrapping_ops {
    { $(impl $trait:ident::$fn:ident for $vector:ident($($field:ident),*);)* } => { $(
        impl<T> $trait for $vector<T>
        where
            T: $trait,
        {
            type Output = $vector<<T as $trait>::Output>;

            fn $fn(self, rhs: $vector<T>) -> Self::Output {
                $vector { $($field: $trait::$fn(self.$field, rhs.$field)),* }
            }
        }

        impl<'a, T> $trait<$vector<T>> for &'a $vector<T>
        where
            &'a T: $trait<T>,
        {
            type Output = $vector<<&'a T as $trait<T>>::Output>;

            fn $fn(self, rhs: $vector<T>) -> Self::Output {
                $vector { $($field: $trait::$fn(&self.$field, rhs.$field)),* }
            }
        }

        impl<'r, T> $trait<&'r $vector<T>> for $vector<T>
        where
            T: $trait<&'r T>,
        {
            type Output = $vector<<T as $trait<&'r T>>::Output>;

            fn $fn(self, rhs: &'r $vector<T>) -> Self::Output {
                $vector { $($field: $trait::$fn(self.$field, &rhs.$field)),* }
            }
        }

        impl<'a, 'r, T> $trait<&'r $vector<T>> for &'a $vector<T>
        where
            &'a T: $trait<&'r T>,
        {
            type Output = $vector<<&'a T as $trait<&'r T>>::Output>;

            fn $fn(self, rhs: &'r $vector<T>) -> Self::Output {
                $vector { $($field: $trait::$fn(&self.$field, &rhs.$field)),* }
            }
        }
    )* };
}

macro_rules! impl_all {
    { $(impl $vector:ident($($field:ident: $T:ident),*)[$n:expr];)* } => { $(
        impl<T> $vector<T> {
            /// Constructs a vector.
            pub const fn new($($field: T),*) -> $vector<T> {
                $vector { $($field),* }
            }
        }

        impl<T> From<[T; $n]> for $vector<T> {
            fn from(a: [T; $n]) -> $vector<T> {
                let mut iter = a.into_iter();
                $(let $field = iter.next().unwrap();)*
                $vector { $($field),* }
            }
        }

        impl<T> From<($($T),*)> for $vector<T> {
            fn from(t: ($($T),*)) -> $vector<T> {
                let ($($field),*) = t;
                $vector { $($field),* }
            }
        }

        impl<T: Identity> Identity for $vector<T> {
            fn identity() -> $vector<T> {
                $vector { $($field: T::identity()),* }
            }
        }

        impl<T> Into<[T; $n]> for $vector<T> {
            fn into(self) -> [T; $n] {
                [$(self.$field),*]
            }
        }

        impl<T> Into<($($T),*)> for $vector<T> {
            fn into(self) -> ($($T),*) {
                ($(self.$field),*)
            }
        }

        impl<T, F: TryInto<T>> TryFromComposite<$vector<F>> for $vector<T> {
            type Error = F::Error;

            fn try_from_composite(from: $vector<F>) -> Result<$vector<T>, Self::Error> {
                Ok($vector { $($field: from.$field.try_into()?),* })
            }
        }

        impl<'a, T, F> TryFromComposite<&'a $vector<F>> for $vector<T>
        where
            &'a F: TryInto<T>,
        {
            type Error = <&'a F as TryInto<T>>::Error;

            fn try_from_composite(from: &'a $vector<F>) -> Result<$vector<T>, Self::Error> {
                Ok($vector { $($field: (&from.$field).try_into()?),* })
            }
        }

        impl<T: Zero> Zero for $vector<T> {
            fn zero() -> $vector<T> {
                $vector { $($field: T::zero()),* }
            }
        }

        impl_unary_ops! {
            impl Neg::neg for $vector($($field),*);
        }

        impl_scalar_ops! {
            impl Div::div for $vector($($field),*);
            impl Mul::mul for $vector($($field),*);
        }

        impl_scalar_assign_ops! {
            impl DivAssign::div_assign for $vector($($field),*);
            impl MulAssign::mul_assign for $vector($($field),*);
        }

        impl_vector_ops! {
            impl Add::add for $vector($($field),*);
            impl Div::div for $vector($($field),*);
            impl Mul::mul for $vector($($field),*);
            impl Sub::sub for $vector($($field),*);
        }

        impl_vector_assign_ops! {
            impl AddAssign::add_assign for $vector($($field),*);
            impl DivAssign::div_assign for $vector($($field),*);
            impl MulAssign::mul_assign for $vector($($field),*);
            impl SubAssign::sub_assign for $vector($($field),*);
        }

        impl_unary_try_ops! {
            impl TryNeg::try_neg for $vector($($field),*);
        }

        impl_scalar_try_ops! {
            impl TryDiv::try_div for $vector($($field),*);
            impl TryMul::try_mul for $vector($($field),*);
        }

        impl_vector_try_ops! {
            impl TryAdd::try_add for $vector($($field),*);
            impl TryDiv::try_div for $vector($($field),*);
            impl TryMul::try_mul for $vector($($field),*);
            impl TrySub::try_sub for $vector($($field),*);
        }

        impl_scalar_wrapping_ops! {
            impl WrappingMul::wrapping_mul for $vector($($field),*);
        }

        impl_vector_wrapping_ops! {
            impl WrappingAdd::wrapping_add for $vector($($field),*);
            impl WrappingMul::wrapping_mul for $vector($($field),*);
            impl WrappingSub::wrapping_sub for $vector($($field),*);
        }
    )* };
}

impl_all! {
    impl Vector2(x: T, y: T)[2];
    impl Vector3(x: T, y: T, z: T)[3];
    impl Vector4(x: T, y: T, z: T, w: T)[4];
}
