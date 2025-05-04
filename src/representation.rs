//! Internal representation of `Utf8Char`

mod enums;

use core::{mem, ptr};

/// An implementation of `codepoint_len` that depends on bmi/tzcnt to be fast
#[expect(clippy::cast_possible_truncation, reason = "leading_ones is weird, the max value is 8, no truncation")]
pub(crate) const fn codepoint_len_bmi(byte: u8) -> u8 {
    (byte.leading_ones().saturating_sub(1) + 1) as u8
}

/// An implementation of `codepoint_len` that should be fast on all architectures
pub(crate) const fn codepoint_len_lut(byte: u8) -> u8 {
    const LUT_SIZE: usize = 16;

    // rely on utf8 that first 4 bits can be used to build a lookup table of the length
    const LUT: [u8; LUT_SIZE] = {
        let mut i = 0;
        let mut arr = [0; LUT_SIZE];
        while i < LUT_SIZE {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "wont truncate, const contexts cant use try_from"
            )]
            {
                arr[i] = codepoint_len_bmi((i as u8) << 4);
            }
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
        let u8c = crate::Utf8Char::from_char(c);
        let first = u8c.0.first_byte().0;

        let control = u8::try_from(c.len_utf8()).expect("within 1..=4");

        assert_eq!(codepoint_len_bmi(first as u8), control);
        assert_eq!(codepoint_len_lut(first as u8), control);
        assert_eq!(u8c.len_utf8(), control);
    });
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
#[expect(
    dead_code,
    reason = "we transmute into/outof these values, rust cant see it"
)]
#[expect(
    clippy::missing_docs_in_private_items,
    reason = "its 1..=4, each variant is unspecial"
)]
/// An enum representing all of the valid lengths of a utf8 encoded codepoint
pub(crate) enum EncodedLength {
    One = 1,
    Two,
    Three,
    Four,
}

/// Transmutes a mut reference from T to U
///
/// # Safety
/// Requires the same safety assertions that `core::mem::transmute` does, as this is a safety
/// wrapper around `core::mem::transmute` (for `&mut T` to `&mut U`, instead of the traditional T to
/// U)
#[expect(
    clippy::needless_lifetimes,
    reason = "This is an unsafe code wrapper, the explicitness is its purpose"
)]
const unsafe fn trans_mut<'a, T, U>(v: &'a mut T) -> &'a mut U {
    // SAFETY: Caller is abiding by transmute contract when calling this function
    unsafe { core::mem::transmute(v) }
}

/// The first and potentially only byte of a utf8 encoded codepoint
// Safety invariant: matches only valid utf8 first byte encodings (documented in representation/enums.rs)
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Utf8FirstByte(pub(crate) enums::Utf8FirstByte);

impl Utf8FirstByte {
    /// Constructs a new `Utf8FirstByte` from an arbitrary u8
    ///
    /// # Safety
    /// value passed must be a valid utf8 encoded characters first byte
    pub(crate) const unsafe fn new(b: u8) -> Self {
        // SAFETY: Caller asserts that byte is in the valid utf8 encoded characters firstbyte range
        Self(unsafe { mem::transmute::<u8, enums::Utf8FirstByte>(b) })
    }

    /// Returns the total length of the encoded character that this `Utf8FirstByte` indicates
    pub(crate) const fn codepoint_len(self) -> EncodedLength {
        let len: u8 = codepoint_len_lut(self.0 as u8);

        // SAFETY: Utf8CharEncodedLength is repr(u8), Utf8Char::codepoint_len will return 1..=4 for
        // any valid utf8 codepoint first byte, which this type stores as a safety invariant
        unsafe { mem::transmute::<u8, EncodedLength>(len) }
    }
}

#[repr(C)]
// NOTE: Eq/Ord rely on the representation guarantee that padding bytes are set to `TAG_CONTINUATION`
#[derive(Copy, Clone, Eq, PartialEq)]
/// Internal representation of a `Utf8Char` with an unsafe API
pub(crate) struct Utf8CharInner(Utf8FirstByte, [enums::Utf8ContByte; 3]);

impl Utf8CharInner {
    /// Returns the length of the char as encoded in utf8
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
    /// byte 1 nicherepr: `(u8 is ..0b1111_1000)` (`NonMaxU8`) AND NOT `0b1000_0000..=0b1100_0001`
    /// byte 2..=4 nicherepr: `(u8 is TAG_CONTINUATION..=0b10_11_1111)`
    pub(crate) const unsafe fn from_utf8char_array(arr: [u8; 4]) -> Self {
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
    pub(crate) const unsafe fn first_byte_mut(&mut self) -> &mut u8 {
        // SAFETY: Utf8FirstByte is repr(u8), caller promises to abide by contract which includes
        // staying within valid states
        unsafe { trans_mut::<enums::Utf8FirstByte, u8>(&mut self.0 .0) }
    }

    /// Returns mutable array reference to entire `Utf8CharInner` repr as a `&mut [u8; 4]`
    /// # Safety
    /// The array *must never* be mutated to a state where it does not follow the utf8char repr as
    /// defined by the safety documentation of [`Self::from_utf8char_array`].
    /// This includes "paired mutations", where one mutation sets an invalid state and a later
    /// mutation brings it back to validity: that is UB. Prefer to do mutations to an array copy
    /// and store once in such cases.
    pub(crate) const unsafe fn total_repr_mut(&mut self) -> &mut [u8; 4] {
        // SAFETY: this type is repr(C) and is a subset of [u8; 4]
        // the caller agrees to not ever store an invalid repr
        unsafe { &mut *ptr::from_mut(self).cast::<[u8; 4]>() }
    }
}

// we need to implement these manually because the optimizer started giving up on the enum
// representation
impl Ord for Utf8CharInner {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_array().cmp(other.as_array())
    }
}

impl PartialOrd for Utf8CharInner {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
