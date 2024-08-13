#![no_std]
// TODO enable
//#![warn(clippy::pedantic)]
//#![warn(missing_docs, clippy::missing_docs_in_private_items)]

use core::{
    fmt::{self, Write},
    hint, mem,
    ops::Deref,
};

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

    const fn byte_len(&self) -> u8 {
        let len = Self::codepoint_len(self.0[0]);

        if !(1 <= len && len <= 4) {
            // SAFETY: codepoint_len will always return 1..=4 for valid utf8, which Utf8Char is
            // always assumed to be
            unsafe { hint::unreachable_unchecked() };
        }

        len
    }

    /// returns a Utf8Char from the first char of a passed str
    ///
    /// # Panics
    /// Panics if length of string is 0, this may be promoted to a const panic when used in const
    /// contexts
    const fn first_char(s: &str) -> Self {
        if s.is_empty() {
            panic!("Utf8Char::first_char called with empty string");
        }

        let b = s.as_bytes();

        let len = Self::codepoint_len(b[0]);

        assert!(s.len() >= len as usize && len <= 4);

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

    pub fn from_char(v: char) -> Self {
        let mut out = [0xff; 4];

        v.encode_utf8(&mut out);

        // NOTE: we are a safety invariant, Utf8Char must always be valid utf8
        // we rely on char::encode_utf8 encoding the char in the buffer
        Self(out)
    }

    pub fn as_char_bad(&self) -> char {
        unsafe { self.deref().chars().next().unwrap_unchecked() } 
    }

    pub const fn as_char(&self) -> char {
        // TODO: replace entire thing with stdlib when its const stable

        const B6: u8 = 0b11_11_11;

        let ch = match self.byte_len() {
            // case: 1 byte is always ascii
            1 => self.0[0] as u32,
            2 => {
                // 5 bits, shift 6 for following byte
                let b1 = ((self.0[0] & 0b11111) as u32) << 6;
                // 6 bits, shift none as we are the last byte
                let b2 = (self.0[1] & B6) as u32;

                b1 | b2
            }
            3 => {
                // 4 bits, shift 12 for following 2 bytes
                let b1 = ((self.0[0] & 0b1111) as u32) << 12;
                // 6 bits, shift 6 for following byte
                let b2 = ((self.0[1] & B6) as u32) << 6;
                // 6 bits, shift none as we are last
                let b3 = (self.0[2] & B6) as u32;

                b1 | b2 | b3
            }
            4 => {
                // 3 bits, shift 18 for following 3 bytes
                let b1 = ((self.0[0] & 0b111) as u32) << 18;
                // 6 bits, shift 12 for following 2 bytes
                let b2 = ((self.0[1] & B6) as u32) << 12;
                // 6 bits, shift 6 for following byte
                let b3 = ((self.0[2] & B6) as u32) << 6;
                // 6 bits, shift none as we are last
                let b4 = (self.0[3] & B6) as u32;

                b1 | b2 | b3 | b4
            }

            _ => panic!("unreachable: utf8 codepoints must be of length 1..=4"),
        };

        // add our own check as we are not using from_u32_unchecked; it is not const
        debug_assert!(char::from_u32(ch).is_some());

        // SAFETY: Utf8Char must always be valid utf8 so this must always be valid
        unsafe { mem::transmute(ch) }
    }
}

impl Deref for Utf8Char {
    type Target = str;

    fn deref(&self) -> &str {
        let len = Self::codepoint_len(self.0[0]) as usize;

        // SAFETY: codepoint_len returns 1..=4 for valid utf8
        // we assume Utf8Char is always valid utf8
        let slice = unsafe { self.0.get_unchecked(0..len) };

        // SAFETY: [u8; codepoint_len] of Utf8Char must be valid Utf8
        unsafe { core::str::from_utf8_unchecked(slice) }
    }
}

impl fmt::Debug for Utf8Char {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.as_char(), f)
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

#[test]
fn roundtrip() {
    for ch in '\0'..=char::MAX {
        let mut buf = [0xff; 4];

        let s = ch.encode_utf8(&mut buf);

        let utf8 = Utf8Char::from_char(ch);

        assert_eq!(s, &*utf8);
        assert_eq!(buf, utf8.0);

        assert_eq!(utf8.as_char(), ch);
    }
}
