#![no_std]
// TODO enable
//#![warn(clippy::pedantic)]
//#![warn(missing_docs, clippy::missing_docs_in_private_items)]

use core::{
    fmt, hint,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

/// Unsafely assumes that the boolean passed in is true
///
/// Safety:
/// It is UB to pass false to this function
const unsafe fn assume(b: bool) {
    if !b {
        unsafe { hint::unreachable_unchecked() }
    }
}

#[derive(Copy, Clone)]
pub struct Utf8Char([u8; 4]);

impl Utf8Char {
    /// Returns the length of a UTF-8 encoded codepoint based on the first bytes
    /// encoding (returns 1..=4).
    ///
    /// `byte` must be the first byte of a valid UTF-8 encoded codepoint.
    const fn codepoint_len(byte: u8) -> u8 {
        (byte.leading_ones().saturating_sub(1) + 1) as u8
    }

    pub const fn byte_len(&self) -> u8 {
        let len = Self::codepoint_len(self.0[0]);

        // SAFETY: codepoint_len will always return 1..=4 for valid utf8, which Utf8Char is
        // always assumed to be
        unsafe { assume(1 <= len && len <= 4) };

        len
    }

    /// returns a Utf8Char from the first char of a passed str. Returns None if the string
    /// contained no characters (was empty)
    ///
    /// This can be more performant than calling from_char, as no utf32 -> utf8 conversion need be
    /// performed
    pub const fn from_first_char(s: &str) -> Option<Self> {
        // this method is provably correct for all unicode characters: it is tested below
        if s.is_empty() {
            return None;
        }

        let b = s.as_bytes();

        let len = Self::codepoint_len(b[0]);

        // SAFETY: codepoint len always returns 1..=4 so len must be less than or equal to 4
        // string length must be greater than or equal to codepoint_len if it is encoded as valid
        // utf8 (a safety invariant of str)
        unsafe { assume(s.len() >= len as usize && len <= 4) };

        // panic safety: we assume &str is valid utf8
        Some(match len {
            // NOTE: We are a safety invariant, the [u8; 4] in Utf8Char must be valid utf8
            // 0xff is used for padding as it is invalid utf8
            1 => Self([b[0], 0xff, 0xff, 0xff]),
            2 => Self([b[0], b[1], 0xff, 0xff]),
            3 => Self([b[0], b[1], b[2], 0xff]),
            4 => Self([b[0], b[1], b[2], b[3]]),
            _ => panic!("unreachable: utf8 codepoints must be of length 1..=4"),
        })
    }

    pub const fn from_char(code: char) -> Self {
        // this method is provably correct for all unicode characters: it is tested below
        // this entire function is a const modified copy of the implementation of char::encode_utf8 in
        // core/char/methods.rs
        // TODO(ultrabear): replace with encode_utf8 when const mut refs are stable (and
        // encode_utf8 is const stable)

        const TAG_CONT: u8 = 0b1000_0000;
        const TAG_TWO: u8 = 0b1100_0000;
        const TAG_THREE: u8 = 0b1110_0000;
        const TAG_FOUR: u8 = 0b1111_0000;

        let mut out = [0xff; 4];

        let len = code.len_utf8();

        let code = code as u32;

        match len {
            1 => out[0] = code as u8,
            2 => {
                out[0] = (code >> 6 & 0x1F) as u8 | TAG_TWO;
                out[1] = (code & 0x3F) as u8 | TAG_CONT;
            }
            3 => {
                out[0] = (code >> 12 & 0x0F) as u8 | TAG_THREE;
                out[1] = (code >> 6 & 0x3F) as u8 | TAG_CONT;
                out[2] = (code & 0x3F) as u8 | TAG_CONT;
            }
            4 => {
                out[0] = (code >> 18 & 0x07) as u8 | TAG_FOUR;
                out[1] = (code >> 12 & 0x3F) as u8 | TAG_CONT;
                out[2] = (code >> 6 & 0x3F) as u8 | TAG_CONT;
                out[3] = (code & 0x3F) as u8 | TAG_CONT;
            }
            _ => panic!("unreachable: len_utf8 must always return 1..=4"),
        }

        // NOTE: we are a safety invariant, Utf8Char must always be valid utf8
        // we rely on our copy paste of char::encode_utf8 encoding the char in the buffer
        Self(out)
    }

    pub const fn to_char(&self) -> char {
        // this method is provably correct for all unicode characters: it is tested below
        // this entire function is copied off of the implementation of str::chars() because it is
        // highly performant
        // the only relevant modifications are ones such that it may be fully used in a const
        // context
        // TODO(ultrabear): replace this entire thing with chars().next().unwrap_unchecked() when
        // that is const stable

        // the below is mostly copied from core/src/str/validations.rs
        const B6: u8 = 0b0011_1111;

        const fn utf8_first_byte(byte: u8, width: u32) -> u32 {
            (byte & (0x7F >> width)) as u32
        }

        const fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
            (ch << 6) | (byte & B6) as u32
        }

        let x = self.0[0];

        if x < 128 {
            return x as char;
        }

        let init = utf8_first_byte(x, 2);

        let y = self.0[1];

        let mut ch = utf8_acc_cont_byte(init, y);

        if x >= 0xE0 {
            let z = self.0[2];

            let y_z = utf8_acc_cont_byte((y & B6) as u32, z);

            ch = init << 12 | y_z;

            if x >= 0xF0 {
                let w = self.0[3];
                ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
            }
        }

        debug_assert!(char::from_u32(ch).is_some());

        // SAFETY: Utf8Char must always be valid utf8 so this must always be valid
        unsafe { mem::transmute(ch) }
    }

    pub fn as_str(&self) -> &str {
        let len = self.byte_len() as usize;

        let slice = &self.0[..len];

        // SAFETY: [u8; byte_len] of Utf8Char must be valid Utf8
        unsafe { core::str::from_utf8_unchecked(slice) }
    }

    // TODO document multi char behavior
    pub fn as_mut_str(&mut self) -> &mut str {
        let len = self.byte_len() as usize;

        let slice = &mut self.0[..len];

        // SAFETY: [u8; byte_len] of Utf8Char must be valid utf8
        unsafe { core::str::from_utf8_unchecked_mut(slice) }
    }
}

impl Deref for Utf8Char {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl DerefMut for Utf8Char {
    fn deref_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}

impl fmt::Debug for Utf8Char {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.to_char(), f)
    }
}

impl fmt::Display for Utf8Char {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // copied from char's Display implementation, sans utf8 encoding
        if f.width().is_none() && f.precision().is_none() {
            f.write_str(&**self)
        } else {
            f.pad(&**self)
        }
    }
}

struct Utf8CharRef<'a>(NonNull<u8>, PhantomData<&'a str>);

impl<'a> Utf8CharRef<'a> {}

#[test]
#[cfg(miri)]
fn greater_than_stacked() {

    // this is a test that we using a provenance memory model that supports our types
    // if this test fails, this crate is UB

    #[repr(transparent)]
    struct Cursed([u8; 1]);

    impl Cursed {
        fn as_ptr(&self) -> *const u8 {
            self as *const Cursed as *const u8
        }
    }

    let one: &'static str = "abcd";

    let oneref: &'static Cursed = unsafe { &*(one.as_ptr() as *const Cursed) };

    // never deref oneref

    assert_eq!(unsafe { *oneref.as_ptr().add(1) }, b'b');
}

#[test]
fn roundtrip() {
    let mut ctr = 5000;

    for ch in '\0'..=char::MAX {
        ctr -= 1;
        if ctr == 0 {
            return;
        }

        let mut buf = [0xff; 4];

        let s = ch.encode_utf8(&mut buf);

        let utf8_alt = Utf8Char::from_first_char(s).unwrap();

        let utf8 = Utf8Char::from_char(ch);

        let codelen = Utf8Char::codepoint_len(utf8.0[0]);

        assert!(matches!(codelen, 1..=4));

        assert_eq!(codelen as usize, ch.len_utf8());
        assert_eq!(utf8.byte_len() as usize, ch.len_utf8());

        assert_eq!(s, &*utf8);
        assert_eq!(&*utf8_alt, s);
        assert_eq!(buf, utf8.0);

        assert_eq!(utf8.to_char(), ch);
        assert_eq!(utf8_alt.to_char(), ch);
    }
}
