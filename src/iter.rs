//! Implements an Iterator on strings that provide Utf8Char's

use core::{iter::FusedIterator, slice};

use crate::{representation::Utf8FirstByte, Utf8Char};

/// An iterator over a string that yields `Utf8Char`'s
#[derive(Clone, Debug)]
pub struct Utf8CharIter<'slice> {
    /// benchmarked, just using this and its normal api is the fastest way already
    /// dont bother writing some manual pointer magic to try and beat it... dont ask
    inner: slice::Iter<'slice, u8>,
}

impl<'slice> Utf8CharIter<'slice> {
    /// Returns the next byte and advances the backing iterator
    ///
    /// # Safety
    /// There must be at least one byte left in the backing iterator
    unsafe fn next_byte_unchecked(&mut self) -> u8 {
        // SAFETY: Caller asserts there is at least one byte left in the backing iterator
        *unsafe { self.inner.next().unwrap_unchecked() }
    }

    /// Constructs a new `Utf8CharIter` from a string slice, borrowing the slice
    pub fn new(s: &'slice str) -> Self {
        Self {
            inner: s.as_bytes().iter(),
        }
    }

    /// # Safety
    /// There must be at least one more codepoint in the backing utf8 slice
    unsafe fn next_unchecked(&mut self) -> Utf8Char {
        // SAFETY: caller ensures pseudo-slice is not empty; we have a byte available to read
        let first = unsafe { self.next_byte_unchecked() };

        // SAFETY: first is the first byte of a potentially multibyte utf8 encoded character
        let len = unsafe { Utf8FirstByte::new(first).codepoint_len() } as u8;

        // construct dummy Utf8Char for overwriting purposes
        let mut ch = const { Utf8Char::from_char('0') };

        // an array that is logically a Utf8Char, but allows us to do arbitrary mutations
        // we will later overwrite ch.total_repr_mut() in a single copy operation to maintain
        // safety invariants
        let mut arr = *ch.0.as_array();

        // 0th byte is always used in utf8
        arr[0] = first;

        if len > 1 {
            // SAFETY: len>1 so the 1st byte is valid
            arr[1] = unsafe { self.next_byte_unchecked() };
        }

        if len > 2 {
            // SAFETY: len>2 so the 2nd byte is valid
            arr[2] = unsafe { self.next_byte_unchecked() };
        }

        if len > 3 {
            // SAFETY: len>3 so the 3rd byte is valid
            arr[3] = unsafe { self.next_byte_unchecked() };
        }

        // SAFETY: arr matches the invariants of utf8char because it was built from a null utf8char and
        // then had 1..=4 bytes of a single unicode codepoint copied, leaving padding intact
        unsafe {
            *ch.0.total_repr_mut() = arr;
        }

        ch
    }

    fn is_empty(&self) -> bool {
        self.inner.as_slice().is_empty()
    }

    fn as_str(&self) -> &'slice str {
        let slice = self.inner.as_slice();

        // SAFETY: iterator is always aligned to a utf8 boundary and originally came from a string
        unsafe { core::str::from_utf8_unchecked(slice) }
    }
}

impl Iterator for Utf8CharIter<'_> {
    type Item = Utf8Char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            // SAFETY: we have checked that the backing array is not empty
            Some(unsafe { self.next_unchecked() })
        }
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.as_str().chars().count()
    }
}

impl FusedIterator for Utf8CharIter<'_> {}

#[test]
fn allstring() {
    use itertools::Itertools;

    let allchars = (char::MIN..=char::MAX).collect::<alloc::string::String>();

    let utf8chars = Utf8CharIter::new(&allchars);

    let chars = allchars.chars();

    assert_eq!(utf8chars.clone().count(), chars.clone().count());

    utf8chars.zip_eq(chars).for_each(|(u8c, c)| {
        assert_eq!(u8c.to_char(), c);
    })
}
