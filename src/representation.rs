//! Internal representation of `Utf8Char`

use core::{mem, num::NonZeroU8, ptr};

/// An implementation of codepoint_len that depends on bmi/tzcnt to be fast
pub(crate) const fn codepoint_len_bmi(byte: u8) -> u8 {
    (byte.leading_ones().saturating_sub(1) + 1) as u8
}

/// An implementation of codepoint_len that should be fast on all architectures
pub(crate) const fn codepoint_len_lut(byte: u8) -> u8 {
    const LUT_SIZE: usize = 16;

    // rely on utf8 that first 4 bits can be used to build a lookup table of the length
    const LUT: [u8; LUT_SIZE] = {
        let mut i = 0;
        let mut arr = [0; LUT_SIZE];
        while i < LUT_SIZE {
            arr[i] = codepoint_len_bmi((i as u8) << 4);
            i += 1;
        }
        arr
    };
    LUT[(byte >> 4) as usize]
}

#[test]
fn identical_codepoint_len() {
    use crate::tests;

    use rayon::iter::ParallelIterator;

    tests::all_chars().for_each(|c| {
        let first = crate::Utf8Char::from_char(c).0.first_byte().0;

        let control = c.len_utf8() as u8;

        assert_eq!(codepoint_len_bmi(first), control);
        assert_eq!(codepoint_len_lut(first), control);
    });
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
#[expect(dead_code, reason = "we transmute into/outof these values, rust cant see it")]
pub(crate) enum EncodedLength {
    One = 1,
    Two,
    Three,
    Four,
}

/// The first and potentially only byte of a utf8 encoded codepoint
// Safety invariant: `u8 is 0..0xff` & `u8 is only a firstbyte of a utf8 codepoint`
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Utf8FirstByte(pub(crate) u8);

impl Utf8FirstByte {
    /// Constructs a new Utf8CharFirstByte from an arbitrary u8
    ///
    /// # Safety
    /// value passed must be a valid utf8 encoded characters first byte
    pub(crate) const unsafe fn new(b: u8) -> Self {
        Self(b)
    }

    pub(crate) const fn codepoint_len(self) -> EncodedLength {
        let len: u8 = codepoint_len_lut(self.0);

        // SAFETY: Utf8CharEncodedLength is repr(u8), Utf8Char::codepoint_len will return 1..=4 for
        // any valid utf8 codepoint first byte, which this type stores as a safety invariant
        unsafe { mem::transmute(len) }
    }
}

#[repr(C)]
// NOTE: Eq/Ord rely on the representation guarantee that padding bytes are set to `TAG_CONTINUATION`
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
/// Internal representation of a `Utf8Char` with an unsafe API
pub(crate) struct Utf8CharInner(Utf8FirstByte, [NonZeroU8; 3]);

impl Utf8CharInner {
    pub(crate) const fn len_utf8(self) -> EncodedLength {
        self.0.codepoint_len()
    }

    /// Constructs a `Utf8CharInner` from an array of utf8char formatted data
    /// (utf8char formatted data is utf8 where padding bytes are `TAG_CONTINUATION`, described below)
    /// # Safety
    /// The following representation must be upheld:
    /// Canonical representation of `Utf8Char` (logical and safety invariants
    /// rely on this, rust cannot fully express it):
    /// 1 byte: utf8, `[TAG_CONTINUATION; 3]`
    /// 2 bytes: [utf8; 2], `[TAG_CONTINUATION; 2]`
    /// 3 bytes: [utf8; 3], `TAG_CONTINUATION`
    /// 4 bytes: [utf8; 4]
    /// byte 1 nicherepr: `(u8 is ..0xFF)` (`NonMaxU8`)
    /// byte 2..=4 nicherepr: `(u8 is TAG_CONTINUATION..=0b10_11_1111)`
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
        unsafe { &*ptr::from_ref(self).cast::<[u8; 4]>() }
    }

    /// Returns first byte which is always dataful
    pub(crate) const fn first_byte(self) -> Utf8FirstByte {
        self.0
    }

    /// Performs an equality check that is compile time compatible
    ///
    /// NOT FOR CRYPTOGRAPHIC PURPOSES
    pub(crate) const fn const_eq(self, other: Self) -> bool {
        // NOTE: this relies on the representation guarantee that padding bytes are set to TAG_CONTINUATION
        let this = u32::from_ne_bytes(*self.as_array());
        let other = u32::from_ne_bytes(*other.as_array());

        this == other
    }

    /// Returns mutable reference to first byte
    /// # Safety
    /// - The first byte must never be illegal as the first byte of a utf8 codepoint
    /// - The first byte must follow the first byte requirements of validity defined by the safety
    ///   documentation of [`Self::from_utf8char_array`]
    /// - The first byte must never change its "data portion" if doing so would result in an illegal utf8
    ///   codepoint across the entire `Utf8CharInner`
    /// - The first byte must never change its size tag (i/e a len: 1 first byte must still encode
    ///   len: 1)
    ///
    /// If you want to change the entire `Utf8CharInner`, use [`total_repr_mut`][Self::total_repr_mut]
    pub(crate) const unsafe fn first_byte_mut(&mut self) -> &mut Utf8FirstByte {
        &mut self.0
    }

    /// Returns mutable array reference to entire `Utf8CharInner` repr as a `&mut [u8; 4]`
    /// # Safety
    /// The array *must never* be mutated to a state where it does not follow the utf8char repr as
    /// defined by the safety documentation of [`Self::from_utf8char_array`].
    /// This includes "paired mutations", where one mutation sets an invalid state and a later
    /// mutation brings it back to validity: that is UB. Prefer to do mutations to an array copy
    /// and store once in such cases.
    #[expect(dead_code, reason = "we may want this one day")]
    pub(crate) const unsafe fn total_repr_mut(&mut self) -> &mut [u8; 4] {
        // SAFETY: this type is repr(C) and is a subset of [u8; 4]
        // the caller agrees to not ever store an invalid repr
        unsafe { &mut *ptr::from_mut(self).cast::<[u8; 4]>() }
    }
}
