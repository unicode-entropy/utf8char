//! Implements an Iterator on strings that provide Utf8Char's

use core::{fmt, iter::FusedIterator, slice};

use crate::{representation::Utf8FirstByte, Utf8Char, Utf8CharInner, TAG_CONTINUATION};

/// Returns whether a given utf8 byte is a continuation byte
fn is_continuation(b: u8) -> bool {
    const TAG_MASK: u8 = 0b1100_0000;

    (b & TAG_MASK) == TAG_CONTINUATION
}

/// An iterator over a string that yields `Utf8Char`'s
#[derive(Clone)]
pub struct Utf8CharIter<'slice> {
    /// benchmarked, just using this and its normal api is the fastest way already
    /// dont bother writing some manual pointer magic to try and beat it... dont ask
    inner: slice::Iter<'slice, u8>,
}

impl fmt::Debug for Utf8CharIter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Utf8CharIter(")?;
        f.debug_list().entries(self.clone()).finish()?;
        write!(f, ")")
    }
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

    /// Returns the next byte from the back of the iterator
    ///
    /// # Safety
    /// There must be at least one byte left in the backing iterator
    unsafe fn next_byte_back_unchecked(&mut self) -> u8 {
        // SAFETY: Caller asserts there is at least one byte left in the backing iterator
        *unsafe { self.inner.next_back().unwrap_unchecked() }
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

    /// Returns the next Utf8Char from the back of the backing slice
    ///
    /// # Safety
    /// There must be at least one char available in the backing slice
    unsafe fn next_back_unchecked(&mut self) -> Utf8Char {
        let mut arr = [0; 4];
        let mut len = 1;

        while len <= 4 {
            // SAFETY: len is 1..=4, 4-1 is 3, 4-4 is 0, no underflow occurs
            let idx = unsafe { 4usize.unchecked_sub(len) };
            // SAFETY: if previous iteration was continuation byte there is at least one more byte
            // to read, otherwise on first iteration caller ensures there is at least one byte to
            // read
            arr[idx] = unsafe { self.next_byte_back_unchecked() };

            if !is_continuation(arr[idx]) {
                break;
            }

            len += 1;
        }

        let mut bits = u32::from_be_bytes(arr);

        // SAFETY: len is 1..=4, will not underflow, generates 0..=3, 3*8 is 24, will not overflow
        // shl to align utf8 to start
        bits <<= unsafe { 4usize.unchecked_sub(len).unchecked_mul(8) };

        // apply TAG_CONTINUATION or'ng to make valid utf8charinner representation
        bits |=
            const { u32::from_be_bytes([0, TAG_CONTINUATION, TAG_CONTINUATION, TAG_CONTINUATION]) };

        // SAFETY: bits matches representation of utf8charinner with padding bytes and all other bytes
        // being part of a single utf8 encoded character
        Utf8Char(unsafe { Utf8CharInner::from_utf8char_array(bits.to_be_bytes()) })
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
        // std has an optimized implementation for us to take advantage of
        // optimizes to no overhead
        self.as_str().chars().count()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // use the stdlib implementation, optimizes to no overhead
        self.as_str().chars().size_hint()
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl DoubleEndedIterator for Utf8CharIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.next_back_unchecked() })
        }
    }
}

impl FusedIterator for Utf8CharIter<'_> {}

/// A convenience trait to make able the ability to call .utf8_chars() on a string just like
/// .chars()
pub trait IntoUtf8Chars {
    /// Returns a Utf8CharIter over the string
    fn utf8_chars<'s>(&'s self) -> Utf8CharIter<'s>;
}

impl IntoUtf8Chars for str {
    fn utf8_chars<'s>(&'s self) -> Utf8CharIter<'s> {
        Utf8CharIter::new(self)
    }
}

#[test]
fn allstring() {
    use itertools::Itertools;

    #[cfg(not(miri))]
    let allchars = (char::MIN..=char::MAX).collect::<alloc::string::String>();
    #[cfg(miri)]
    let allchars = (char::MIN..=char::MAX)
        .take(10_000)
        .collect::<alloc::string::String>();

    let utf8chars = Utf8CharIter::new(&allchars);

    let chars = allchars.chars();

    assert_eq!(utf8chars.clone().count(), chars.clone().count());

    utf8chars
        .clone()
        .zip_eq(chars.clone())
        .for_each(|(u8c, c)| {
            assert_eq!(u8c.to_char(), c);

            // ensures bitrepr compatibility is upheld
            assert_eq!(Utf8Char::from_char(u8c.to_char()), u8c);
        });

    utf8chars.rev().zip_eq(chars.rev()).for_each(|(u8c, c)| {
        assert_eq!(u8c.to_char(), c);

        // ensures bitrepr compatibility is upheld
        assert_eq!(Utf8Char::from_char(u8c.to_char()), u8c);
    });
}
