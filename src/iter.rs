//! Implements an Iterator on strings that provide Utf8Char's

use core::{
    marker::PhantomData,
    mem,
    ptr::{self, NonNull},
};

use crate::Utf8Char;

/// An iterator over a string that yields `Utf8Char`'s
// modeled after slice::Iter
pub struct Utf8CharIter<'slice> {
    /// start pointer that is incremented on iter::next
    ptr: NonNull<u8>,
    /// end pointer that is beyond the provenance range
    end: NonNull<u8>,

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
        unsafe { mem::transmute(len) }
    }
}

impl<'slice> Utf8CharIter<'slice> {
    /// Fills a buffer with 1..=4 bytes from the backing slice, advancing the iterator
    ///
    /// # Safety
    /// There must be at least `n` bytes available in the backing iterator
    unsafe fn fill_buf(&mut self, buf: &mut [u8; 4], n: Utf8CharEncodedLength) {
        // SAFETY: caller has ensured backing iterator has enough bytes to fill the requested
        // amount of 1..=4 and advance the iterator by the same amount
        //
        // this is a very idiomatic way to copy these bytes, its possibly not very efficient but it
        // is trivially correct
        unsafe {
            ptr::copy_nonoverlapping(self.ptr.as_ptr(), buf.as_mut_ptr(), n as usize);
            self.ptr = self.ptr.add(n as usize);
        }
    }

    /// Constructs a new `Utf8CharIter` from a string slice, borrowing the slice
    fn new(s: &'slice str) -> Self {
        // SAFETY: String reference cannot be null
        let ptr = unsafe { NonNull::new_unchecked(s.as_ptr().cast_mut()) };

        // SAFETY: It is always sound to add the length of an allocation to its start pointer, see
        // ptr::add docs for clarification
        let end = unsafe { ptr.add(s.len()) };

        // lexically express captured lifetime
        let lifetime = PhantomData::<&'slice str>;

        Self { ptr, end, lifetime }
    }

    /// # Safety
    /// There must be at least one more codepoint in the backing utf8 slice
    unsafe fn next_unchecked(&mut self) -> Utf8Char {
        // SAFETY: caller ensures pseudo-slice is not empty; we have provenance over the byte behind ptr
        let first = unsafe { self.ptr.read() };

        // SAFETY: first is the first byte of a potentially multibyte utf8 encoded character
        let len = unsafe { Utf8CharFirstByte::new(first).codepoint_len() };

        // construct dummy Utf8Char for overwriting purposes
        let mut ch = const { Utf8Char::from_char('0') };

        // an array that is logically a Utf8Char, but allows us to do arbitrary mutations
        // we will later overwrite ch.total_repr_mut() in a single copy operation to maintain
        // safety invariants
        let mut arr = *ch.0.as_array();

        // SAFETY: There are at least len bytes available in the backing iterator because the utf8
        // codepoint's first byte indicated the length and we are iterating over valid utf8
        unsafe { self.fill_buf(&mut arr, len) };

        // SAFETY: arr matches the invariants of utf8char because it was built from a null utf8char and
        // then had 1..=4 bytes of a single unicode codepoint copied, leaving padding intact
        unsafe {
            *ch.0.total_repr_mut() = arr;
        }

        ch
    }
}
