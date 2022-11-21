/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/// Divides integers, returning the ceiling of the result rather than the floor.
pub fn div_ceil(n: usize, d: usize) -> usize {
    let mut r = n / d;
    if n % d != 0 {
        r += 1;
    }
    r
}
