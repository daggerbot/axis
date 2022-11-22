/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

macro_rules! impl_scalar_ops {
    { $(impl $trait:ident::$fn:ident for $color:ident($field:ident);)* } => { $(
        impl<T> $trait<T> for $color<T>
        where
            T: $trait,
        {
            type Output = $color<<T as $trait>::Output>;

            fn $fn(self, rhs: T) -> Self::Output {
                $color { $field: $trait::$fn(self.$field, rhs) }
            }
        }

        impl<'a, T> $trait<T> for &'a $color<T>
        where
            &'a T: $trait<T>,
        {
            type Output = $color<<&'a T as $trait<T>>::Output>;

            fn $fn(self, rhs: T) -> Self::Output {
                $color { $field: $trait::$fn(&self.$field, rhs) }
            }
        }

        impl<'r, T> $trait<&'r T> for $color<T>
        where
            T: $trait<&'r T>,
        {
            type Output = $color<<T as $trait<&'r T>>::Output>;

            fn $fn(self, rhs: &'r T) -> Self::Output {
                $color { $field: $trait::$fn(self.$field, rhs) }
            }
        }

        impl<'a, 'r, T> $trait<&'r T> for &'a $color<T>
        where
            &'a T: $trait<&'r T>,
        {
            type Output = $color<<&'a T as $trait<&'r T>>::Output>;

            fn $fn(self, rhs: &'r T) -> Self::Output {
                $color { $field: $trait::$fn(&self.$field, rhs) }
            }
        }
    )* };

    { $(impl $trait:ident::$fn:ident for $color:ident($($field:ident),*);)* } => { $(
        impl<T> $trait<T> for $color<T>
        where
            T: Copy + $trait,
        {
            type Output = $color<<T as $trait>::Output>;

            fn $fn(self, rhs: T) -> Self::Output {
                $color { $($field: $trait::$fn(self.$field, rhs)),* }
            }
        }

        impl<'a, T> $trait<T> for &'a $color<T>
        where
            T: Copy,
            &'a T: $trait<T>,
        {
            type Output = $color<<&'a T as $trait<T>>::Output>;

            fn $fn(self, rhs: T) -> Self::Output {
                $color { $($field: $trait::$fn(&self.$field, rhs)),* }
            }
        }

        impl<'r, T> $trait<&'r T> for $color<T>
        where
            T: $trait<&'r T>,
        {
            type Output = $color<<T as $trait<&'r T>>::Output>;

            fn $fn(self, rhs: &'r T) -> Self::Output {
                $color { $($field: $trait::$fn(self.$field, rhs)),* }
            }
        }

        impl<'a, 'r, T> $trait<&'r T> for &'a $color<T>
        where
            &'a T: $trait<&'r T>,
        {
            type Output = $color<<&'a T as $trait<&'r T>>::Output>;

            fn $fn(self, rhs: &'r T) -> Self::Output {
                $color { $($field: $trait::$fn(&self.$field, rhs)),* }
            }
        }
    )* };
}

macro_rules! impl_scalar_assign_ops {
    { $(impl $trait:ident::$fn:ident for $color:ident($field:ident);)* } => { $(
        impl<T> $trait<T> for $color<T>
        where
            T: $trait,
        {
            fn $fn(&mut self, rhs: T) {
                $trait::$fn(&mut self.$field, rhs);
            }
        }

        impl<'r, T> $trait<&'r T> for $color<T>
        where
            T: $trait<&'r T>,
        {
            fn $fn(&mut self, rhs: &'r T) {
                $trait::$fn(&mut self.$field, rhs);
            }
        }
    )* };

    { $(impl $trait:ident::$fn:ident for $color:ident($($field:ident),*);)* } => { $(
        impl<T> $trait<T> for $color<T>
        where
            T: Copy + $trait,
        {
            fn $fn(&mut self, rhs: T) {
                $($trait::$fn(&mut self.$field, rhs);)*
            }
        }

        impl<'r, T> $trait<&'r T> for $color<T>
        where
            T: $trait<&'r T>,
        {
            fn $fn(&mut self, rhs: &'r T) {
                $($trait::$fn(&mut self.$field, rhs);)*
            }
        }
    )* };
}
