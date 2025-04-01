//! Implements an Iterator on strings that provide Utf8Char's

use core::{
    hint::assert_unchecked,
    marker::PhantomData,
    ptr::{self, NonNull},
};

use crate::Utf8Char;

/// An iterator over a string that yields `Utf8Char`'s
// modeled after slice::Iter
pub struct Utf8CharIter<'slice> {
    /// start pointer that is incremented on iter::next
    ptr: NonNull<u8>,
    /// end pointer that is beyond the provenance range
    end: *const u8,

    /// Phantom lifetime of our backing string
    lifetime: PhantomData<&'slice str>,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum Utf8CharEncodedLength {
    One = 1,
    Two,
    Three,
    Four,
}

/// The first and potentially only byte of a utf8 encoded codepoint
// Safety invariant: `u8 is 0..0xff` & `u8 is only a firstbyte of a utf8 codepoint`
#[repr(transparent)]
struct Utf8CharFirstByte(u8);

impl Utf8CharFirstByte {
    /// Constructs a new `Utf8CharFirstByte` from a `Utf8Char`'s first byte
    fn from_utf8char(c: Utf8Char) -> Self {
        Self(c.0.first_byte())
    }

    /// Constructs a new Utf8CharFirstByte from an arbitrary u8
    ///
    /// # Safety
    /// value passed must be a valid utf8 encoded characters first byte
    unsafe fn new(b: u8) -> Self {
        Self(b)
    }

    fn codepoint_len(self) -> Utf8CharEncodedLength {
        let len: u8 = Utf8Char::codepoint_len(self.0);

        // SAFETY: Utf8CharEncodedLength is repr(u8), Utf8Char::codepoint_len will return 1..=4 for
        // any valid utf8 codepoint first byte, which this type stores as a safety invariant
        unsafe { core::mem::transmute(len) }
    }
}

impl<'slice> Utf8CharIter<'slice> {
    /// Fills a buffer with 1..=4 bytes from the backing slice, advancing the iterator
    ///
    /// # Safety
    /// There must be at least `n` bytes available in the backing iterator
    unsafe fn fill_buf(&mut self, buf: &mut [u8; 4], n: Utf8CharEncodedLength) {}

    /// Constructs a new `Utf8CharIter` from a string slice, borrowing the slice
    fn new(s: &'slice str) -> Self {}

    /// # Safety
    /// There must be at least one more codepoint in the backing utf8 slice
    unsafe fn next_unchecked(&mut self) -> Utf8Char {
        // SAFETY: caller ensures pseudo-slice is not empty; we have provenance over the byte behind ptr
        let len = Utf8Char::codepoint_len(unsafe { self.ptr.read() });

        // SAFETY: Utf8Char::codepoint_len will always return 1..=4 for a utf8 encoded character's first byte
        unsafe { assert_unchecked((1..=4).contains(&len)) };

        // construct dummy Utf8Char for overwriting purposes
        let mut ch = const { Utf8Char::from_char('0') };

        // an array that is logically a Utf8Char, but allows us to do arbitrary mutations
        // we will later overwrite ch.total_repr_mut() in a single copy operation to maintain
        // safety invariants
        let mut arr = *ch.0.as_array();

        // SAFETY: len is derived from the utf8 len of the codepoint, meaning self.ptr has at least
        // that many bytes available for reading, arr has len=4 which is within the 1..=4 range
        // that len inhabits
        //
        // this is a very idiomatic way to copy these bytes, its possibly not very efficient but it
        // is trivially correct
        unsafe {
            ptr::copy_nonoverlapping(self.ptr.as_ptr(), arr.as_mut_ptr(), usize::from(len));
            self.ptr = self.ptr.add(usize::from(len));
        }

        // SAFETY: arr matches the invariants of utf8char because it was built from a null utf8char and
        // then had 1..=4 bytes of a single unicode codepoint copied, leaving padding intact
        unsafe {
            *ch.0.total_repr_mut() = arr;
        }

        ch
    }
}
