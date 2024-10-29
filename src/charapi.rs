//! Containing file for API's that mimic `char` behaviour

use core::fmt;

use crate::representation::Utf8CharInner;

use super::Utf8Char;

impl fmt::Debug for Utf8Char {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.to_char(), f)
    }
}

impl fmt::Display for Utf8Char {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // copied from char's Display implementation, sans utf8 encoding
        if f.width().is_none() && f.precision().is_none() {
            f.write_str(self.as_str())
        } else {
            f.pad(self.as_str())
        }
    }
}

// TODO(ultrabear): implement all of these
// skip if the implementation would be faster as to_char().method()
#[allow(unused)]
impl Utf8Char {
    /// Returns first byte of utf8char, more compact than writing `self.0.first_byte()`
    const fn ascii(self) -> u8 {
        self.0.first_byte()
    }

    /// Const compatible equality method
    const fn const_eq(self, other: Self) -> bool {
        self.0.const_eq(other.0)
    }

    pub const fn eq_ignore_ascii_case(self, other: Self) -> bool {
        self.to_ascii_lowercase()
            .const_eq(other.to_ascii_lowercase())
    }

    pub const fn is_ascii(self) -> bool {
        matches!(self.ascii(), 0..=127)
    }
    pub const fn is_ascii_alphabetic(self) -> bool {
        self.is_ascii_lowercase() | self.is_ascii_uppercase()
    }
    pub const fn is_ascii_alphanumeric(self) -> bool {
        self.is_ascii_alphabetic() | self.is_ascii_digit()
    }
    pub const fn is_ascii_control(&self) -> bool {
        // copied from std char impl; I have no clue what counts
        matches!(self.ascii(), b'\0'..=b'\x1F' | b'\x7F')
    }
    pub const fn is_ascii_digit(&self) -> bool {
        matches!(self.ascii(), b'0'..=b'9')
    }
    pub const fn is_ascii_graphic(&self) -> bool {
        matches!(self.ascii(), b'!'..=b'~')
    }
    pub const fn is_ascii_hexdigit(&self) -> bool {
        matches!(self.ascii(), b'A'..=b'F' | b'a'..=b'f') | self.is_ascii_digit()
    }
    pub const fn is_ascii_lowercase(&self) -> bool {
        matches!(self.ascii(), b'a'..=b'z')
    }
    pub const fn is_ascii_punctuation(&self) -> bool {
        matches!(self.ascii(), b'!'..=b'/' | b':'..=b'@' | b'['..=b'`' | b'{'..=b'~')
    }
    pub const fn is_ascii_uppercase(&self) -> bool {
        matches!(self.ascii(), b'A'..=b'Z')
    }
    pub const fn is_ascii_whitespace(&self) -> bool {
        matches!(self.ascii(), b'\t' | b'\n' | b'\x0C' | b'\r' | b' ')
    }

    pub const fn make_ascii_lowercase(&mut self) {
        *self = self.to_ascii_lowercase();
    }
    pub const fn make_ascii_uppercase(&mut self) {
        *self = self.to_ascii_uppercase();
    }
    pub const fn to_ascii_lowercase(mut self) -> Self {
        if self.is_ascii_uppercase() {
            // SAFETY: we only modify if is_ascii_uppercase is true (len: 1), taking it to another
            // valid ascii value (len: 1)
            let ascii = unsafe { self.0.first_byte_mut() };

            *ascii += b'a' - b'A';
        }

        self
    }
    pub const fn to_ascii_uppercase(mut self) -> Self {
        if self.is_ascii_lowercase() {
            // SAFETY: we only modify if is_ascii_lowercase is true (len: 1), taking it to another
            // valid ascii value (len: 1)
            let ascii = unsafe { self.0.first_byte_mut() };

            *ascii -= b'a' - b'A';
        }

        self
    }

    pub const fn is_digit(self, radix: u8) -> bool {
        self.to_digit(radix).is_some()
    }
    pub const fn to_digit(self, radix: u8) -> Option<u8> {
        // Copied completely from char::to_digit with slight tweaks to support a u8 based api

        // wraps on out of bounds characters
        let mut digit = self.ascii().wrapping_sub(b'0');

        if radix > 10 {
            assert!(radix <= 36, "to_digit: radix is too high (maximum 36)");
            if digit < 10 {
                return Some(digit);
            }

            // Set the 6th bit to ensure ascii is lowercase
            digit = (self.ascii() | 0b10_0000)
                .wrapping_sub(b'a')
                .saturating_add(10);
        }

        if digit < radix {
            Some(digit)
        } else {
            None
        }
    }
}

#[test]
fn charapi_matches() {
    use rayon::iter::ParallelIterator;

    crate::tests::all_chars().for_each(|c| {
        let utf8 = Utf8Char::from_char(c);

        macro_rules! identical {
            ($($fn:ident),+) => {
                $( assert_eq!(utf8.$fn(), c.$fn(), "{utf8:?}:{c:?}"); )+
            };
        }

        identical!(
            is_ascii,
            is_ascii_alphabetic,
            is_ascii_alphanumeric,
            is_ascii_control,
            is_ascii_digit,
            is_ascii_graphic,
            is_ascii_hexdigit,
            is_ascii_lowercase,
            is_ascii_punctuation,
            is_ascii_uppercase,
            is_ascii_whitespace
        );

        assert_eq!(utf8.to_ascii_lowercase().to_char(), c.to_ascii_lowercase());
        assert_eq!(utf8.to_ascii_uppercase().to_char(), c.to_ascii_uppercase());

        let mut newch = utf8;
        newch.make_ascii_lowercase();
        assert_eq!(newch, utf8.to_ascii_lowercase());

        let mut newch = utf8;
        newch.make_ascii_uppercase();
        assert_eq!(newch, utf8.to_ascii_uppercase());

        for radix in 0..=36 {
            assert_eq!(utf8.is_digit(radix), c.is_digit(radix as u32));
            assert_eq!(
                utf8.to_digit(radix),
                c.to_digit(radix as u32).map(|n| n as u8)
            );
        }
    })
}
