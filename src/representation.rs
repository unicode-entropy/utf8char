use core::{mem, num::NonZeroU8};

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct Utf8CharInner(u8, [NonZeroU8; 3]);

impl Utf8CharInner {
    /// Constructs a Utf8CharInner from an array of utf8char formatted data
    /// (utf8char formatted data is utf8 where padding bytes are TAG_CONTINUATION, described below)
    /// # Safety
    /// The following representation must be upheld:
    /// Canonical representation of Utf8Char (logical and safety invariants
    /// rely on this, rust cannot fully express it):
    /// 1 byte: utf8, [TAG_CONTINUATION; 3]
    /// 2 bytes: [utf8; 2], [TAG_CONTINUATION; 2]
    /// 3 bytes: [utf8; 3], TAG_CONTINUATION
    /// 4 bytes: [utf8; 4]
    /// byte 1 nicherepr: (u8 is ..0xFF) (NonMaxU8)
    /// byte 2..=4 nicherepr: (u8 is TAG_CONTINUATION..=0b10_11_1111)
    pub(crate) const unsafe fn from_utf8char_array(arr: [u8; 4]) -> Self {
        // TODO(ultrabear): debug_assume representation guarantees

        // SAFETY: the caller has abided by the safety contract, which is a strict subset of the
        // data allowed in this representation, we are also repr(C) so the layout is the same
        unsafe { mem::transmute(arr) }
    }

    /// Returns repr as an array of u8
    pub(crate) const fn as_array(&self) -> &[u8; 4] {
        // SAFETY: this type is repr(C) and is a subset of [u8; 4]
        // it would be unsafe to allow mutable access as niches
        // could be invalidated, but it is safe to allow immutable access
        unsafe { &*(self as *const Self).cast::<[u8; 4]>() }
    }

    /// Returns first byte which is always dataful
    pub(crate) const fn first_byte(&self) -> u8 {
        self.0
    }
}
