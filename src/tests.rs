//! Module to ease testing with miri and nonmiri

use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};

// all of the step_by numbers are primes

#[cfg(not(miri))]
pub(crate) fn all_chars() -> impl ParallelIterator<Item = char> {
    ('\0'..=char::MAX).into_par_iter()
}

#[cfg(miri)]
pub(crate) fn all_chars() -> impl Iterator<Item = char> {
    ('\0'..=char::MAX).step_by(4_999)
}

#[cfg(not(miri))]
pub(crate) fn all_char_pairs() -> impl ParallelIterator<Item = (char, char)> {
    ('\0'..=char::MAX)
        .step_by(131)
        .cartesian_product(('\0'..=char::MAX).step_by(131))
        .par_bridge()
}

#[cfg(miri)]
pub(crate) fn all_char_pairs() -> impl Iterator<Item = (char, char)> {
    ('\0'..=char::MAX)
        .step_by(10_007)
        .cartesian_product(('\0'..=char::MAX).step_by(10_007))
}
