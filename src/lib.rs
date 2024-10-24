#![no_std]
// TODO(ultrabear): enable
//#![warn(clippy::pedantic)]
//#![warn(missing_docs, clippy::missing_docs_in_private_items)]

use core::{
    borrow::Borrow,
    char,
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    hint,
    ops::{Deref, DerefMut},
};

/// Unsafely assumes that the boolean passed in is true
///
/// # Safety:
/// This function may only be called with the value of true
const unsafe fn assume(b: bool) {
    if !b {
        unsafe { hint::unreachable_unchecked() }
    }
}

/// A single unicode codepoint encoded in utf8.
///
/// This type is like a contemporary `char`:
/// - Debug/Display is has identical output
/// - Represents a single unicode codepoint
/// - Has a size of 4 bytes
/// - Is Copy
///
/// However, being encoded as utf8, you can take a `&str` reference to it, or a `&mut str` reference to
/// it, it is also `Borrow<str>`, and `PartialOrd<str>`.
/// Encoding between a char and utf8 is expensive and branched,
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
    /// This can be more performant than calling from_char, as no complex utf8 conversion will be
    /// performed
    pub const fn from_first_char(s: &str) -> Option<Self> {
        // this method is provably correct for all unicode characters: it is tested below
        // writing it this way forces us to branch if the condition is not met
        let false = s.is_empty() else {
            return None;
        };

        // SAFETY: We have just checked the string is not empty, and returned otherwise
        Some(unsafe { Self::from_first_char_unchecked(s) })
    }

    /// returns a Utf8Char from the first char of a passed non empty string.
    ///
    /// # Safety:
    /// This function must be called with a string that has a length greater than or equal to one.
    pub const unsafe fn from_first_char_unchecked(s: &str) -> Self {
        // SAFETY: the caller must always pass a nonempty string as a safety invariant
        unsafe { assume(s.len() >= 1) };

        let b = s.as_bytes();

        let len = Self::codepoint_len(b[0]);

        // SAFETY: codepoint len always returns 1..=4 on valid utf8 so len must be in that range
        unsafe { assume(1 <= len && len <= 4) };

        // SAFETY: string length must be greater than or equal to codepoint_len if it is encoded as valid
        // utf8 (a safety invariant of str)
        unsafe { assume(s.len() >= len as usize) };

        // panic safety: we assume &str is valid utf8
        match len {
            // NOTE: We are a safety invariant, the [u8; 4] in Utf8Char must be valid utf8
            // 0xff is used for padding as it is invalid utf8
            1 => Self([b[0], 0xff, 0xff, 0xff]),
            2 => Self([b[0], b[1], 0xff, 0xff]),
            3 => Self([b[0], b[1], b[2], 0xff]),
            4 => Self([b[0], b[1], b[2], b[3]]),
            _ => panic!("unreachable: utf8 codepoints must be of length 1..=4"),
        }
    }

    /// Creates a `Utf8Char` from a `char`
    pub const fn from_char(code: char) -> Self {
        // this method is provably correct for all unicode characters: it is tested below
        // this entire function is a const modified copy of the implementation of char::encode_utf8 in
        // core/char/methods.rs
        // FIXME(ultrabear): replace with encode_utf8 when const mut refs are stable (and
        // encode_utf8 is const stable)
        // FIXME(1.83): const_mut_refs and encode_utf8 const stable as of 1.83

        const TAG_CONTINUATION: u8 = 0b1000_0000;
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
                out[1] = (code & 0x3F) as u8 | TAG_CONTINUATION;
            }
            3 => {
                out[0] = (code >> 12 & 0x0F) as u8 | TAG_THREE;
                out[1] = (code >> 6 & 0x3F) as u8 | TAG_CONTINUATION;
                out[2] = (code & 0x3F) as u8 | TAG_CONTINUATION;
            }
            4 => {
                out[0] = (code >> 18 & 0x07) as u8 | TAG_FOUR;
                out[1] = (code >> 12 & 0x3F) as u8 | TAG_CONTINUATION;
                out[2] = (code >> 6 & 0x3F) as u8 | TAG_CONTINUATION;
                out[3] = (code & 0x3F) as u8 | TAG_CONTINUATION;
            }
            _ => panic!("unreachable: len_utf8 must always return 1..=4"),
        }

        // NOTE: we are a safety invariant, Utf8Char must always be valid utf8
        // we rely on our copy paste of char::encode_utf8 encoding the char in the buffer
        Self(out)
    }

    /// Converts a `Utf8Char` to a `char`
    pub const fn to_char(&self) -> char {
        // this method is provably correct for all unicode characters: it is tested below
        // this entire function is copied off of the implementation of str::chars() because it is
        // highly performant
        // the only relevant modifications are ones such that it may be fully used in a const
        // context
        // FIXME(ultrabear): replace this entire thing with chars().next().unwrap_unchecked() when
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
        unsafe { char::from_u32_unchecked(ch) }
    }

    /// Returns a string reference to the codepoint
    pub const fn as_str(&self) -> &str {
        let len = self.byte_len() as usize;

        // SAFETY: byte_len will always return in the range 1..=4
        let slice = unsafe { self.0.split_at_unchecked(len).0 };

        // SAFETY: [u8; byte_len] of Utf8Char must be valid Utf8
        unsafe { core::str::from_utf8_unchecked(slice) }
    }

    /// Returns mutable string reference to codepoint
    ///
    /// Mutations through this method that cause the string to consist of multiple codepoints, i/e
    /// a single 4 byte codepoint being mutated into 4 individual 1 byte codepoints, will not change the
    /// guarantee of `Utf8Char`; The first codepoint in the string will be interpreted as the only
    /// codepoint in the `Utf8Char`, and the length will change to match.
    pub fn as_mut_str(&mut self) -> &mut str {
        let len = self.byte_len() as usize;

        // SAFETY: byte_len will always return in the range 1..=4
        let slice = unsafe { self.0.split_at_mut_unchecked(len).0 };

        // SAFETY: [u8; byte_len] of Utf8Char must be valid utf8
        unsafe { core::str::from_utf8_unchecked_mut(slice) }
    }
}

impl From<char> for Utf8Char {
    fn from(value: char) -> Self {
        Self::from_char(value)
    }
}

impl From<Utf8Char> for char {
    fn from(value: Utf8Char) -> Self {
        value.to_char()
    }
}

impl AsRef<str> for Utf8Char {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for Utf8Char {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<str> for Utf8Char {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

impl PartialEq<Utf8Char> for str {
    fn eq(&self, other: &Utf8Char) -> bool {
        self.eq(other.as_str())
    }
}

impl PartialOrd<str> for Utf8Char {
    fn partial_cmp(&self, other: &str) -> Option<Ordering> {
        Some(self.as_str().cmp(other))
    }
}

impl PartialOrd<Utf8Char> for str {
    fn partial_cmp(&self, other: &Utf8Char) -> Option<Ordering> {
        Some(self.cmp(other.as_str()))
    }
}

impl Hash for Utf8Char {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl Ord for Utf8Char {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for Utf8Char {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.as_str().cmp(other.as_str()))
    }
}

impl PartialEq for Utf8Char {
    fn eq(&self, other: &Self) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl Eq for Utf8Char {}

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

#[cfg(test)]
extern crate alloc;

#[test]
fn roundtrip() {
    for ch in '\0'..=char::MAX {
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

#[test]
fn displays() {
    use alloc::string::String;
    use core::{fmt::Write, write};

    let mut bufutf32 = String::with_capacity(32);
    let mut bufutf8 = String::with_capacity(32);

    for utf32 in '\0'..=char::MAX {
        let mut utf8 = Utf8Char::from_char(utf32);

        // Display
        bufutf8.clear();
        bufutf32.clear();

        write!(bufutf8, "{utf8}").unwrap();
        write!(bufutf32, "{utf32}").unwrap();

        assert_eq!(bufutf8, bufutf32);
        assert_eq!(bufutf8, utf8.as_str());
        assert_eq!(bufutf8, &*utf8.as_mut_str());

        // Debug
        bufutf8.clear();
        bufutf32.clear();

        write!(bufutf8, "{utf8:?}").unwrap();
        write!(bufutf32, "{utf32:?}").unwrap();

        assert_eq!(bufutf8, bufutf32);
    }
}
