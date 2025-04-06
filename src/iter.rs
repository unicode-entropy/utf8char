//! Implements an Iterator on strings that provide Utf8Char's

use core::{
    iter::FusedIterator,
    marker::PhantomData,
    ptr::{self, NonNull},
    slice,
};

use crate::{representation::Utf8FirstByte, Utf8Char};

/// An iterator over a string that yields `Utf8Char`'s
#[derive(Clone, Debug)]
pub struct Utf8CharIter<'slice> {
    inner: slice::Iter<'slice, u8>,
}

/// Crafted so as to optimize into nothing, but give us raw copies of the pointers
fn iter_to_raw<'a>(s: slice::Iter<'a, u8>) -> (NonNull<u8>, NonNull<u8>, PhantomData<&'a str>) {
    let r = s.as_slice().as_ptr_range();

    (
        // SAFETY: slice::Iter pointers cannot be null on a non ZST
        unsafe { NonNull::new_unchecked(r.start.cast_mut()) },
        // SAFETY: slice::Iter pointers cannot be null on a non ZST
        unsafe { NonNull::new_unchecked(r.end.cast_mut()) },
        PhantomData,
    )
}

/// SAFETY: start, end, and lt must be derived from iter_to_raw
unsafe fn raw_to_iter<'a>(
    start: NonNull<u8>,
    end: NonNull<u8>,
    lt: PhantomData<&'a str>,
) -> slice::Iter<'a, u8> {
    let _consumed_in_typesignature = lt;

    // SAFETY: caller promises start and end were derived from iter_to_raw which is in turn backed
    // by a valid slice
    unsafe { slice::from_raw_parts(start.as_ptr(), end.offset_from(start) as usize) }.iter()
}

impl<'slice> Utf8CharIter<'slice> {
    /// Advances the backing iterator by n bytes
    ///
    /// # Safety
    /// There must be at least n bytes available in the backing iterator to be advanced over
    unsafe fn advance_unchecked(&mut self, n: usize) {
        let (mut start, end, lt) = iter_to_raw(self.inner.clone());

        // SAFETY: caller has asserted there are at least n bytes available in backing array
        start = unsafe { start.add(n) };

        // SAFETY: start/end/lt were constructed from iter_to_raw, and start was modified to a
        // still valid state
        self.inner = unsafe { raw_to_iter(start, end, lt) };
    }

    /// Peeks the n'th next byte without advancing the iterator
    ///
    /// # Safety
    /// The backing iterator must have at least N+1 bytes available for reading
    unsafe fn peek_unchecked(&self, n: usize) -> u8 {
        let (read, _, _) = iter_to_raw(self.inner.clone());

        // SAFETY: Caller has asserted there are at least n+1 more bytes left to be read
        unsafe { read.add(n).read() }
    }

    /// Constructs a new `Utf8CharIter` from a string slice, borrowing the slice
    fn new(s: &'slice str) -> Self {
        Self {
            inner: s.as_bytes().iter(),
        }
    }

    /// # Safety
    /// There must be at least one more codepoint in the backing utf8 slice
    unsafe fn next_unchecked(&mut self) -> Utf8Char {
        // SAFETY: caller ensures pseudo-slice is not empty; we have a byte available to read
        let first = unsafe { self.peek_unchecked(0) };

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

        // for each len>n peek_unchecked(n) is valid
        if len > 1 {
            // SAFETY: len>1 so the 1st byte is valid
            arr[1] = unsafe { self.peek_unchecked(1) };
        }

        if len > 2 {
            // SAFETY: len>2 so the 2nd byte is valid
            arr[2] = unsafe { self.peek_unchecked(2) };
        }

        if len > 3 {
            // SAFETY: len>3 so the 3rd byte is valid
            arr[3] = unsafe { self.peek_unchecked(3) };
        }

        // SAFETY: advance by the length we just read, which must be valid
        unsafe { self.advance_unchecked(len as usize) };

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
