//! Containing file for API's that mimic `char` behaviour

use core::fmt;

use super::Utf8Char;

impl fmt::Debug for Utf8Char {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // copying the implementation deemed too
        // much of a maintenance burden for debug printing
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

/// projects ascii is_* methods from u8 ascii methods
macro_rules! project_ascii {
        ($($name:ident),+,) => {
            $(

            #[must_use]
            #[doc=concat!("equivalent to [`char::", stringify!($name), "`] for `Utf8Char`.")]
            pub const fn $name(self) -> bool {
                self.ascii().$name()
            }

            )+
        }


    }

// TODO(ultrabear): implement all of these
// skip if the implementation would be faster as to_char().method()
#[allow(unused)]
impl Utf8Char {
    /// Returns first byte of utf8char, more compact than writing `self.0.first_byte().0`
    #[must_use]
    const fn ascii(self) -> u8 {
        self.0.first_byte().0 as u8
    }

    /// Const compatible equality method
    #[must_use]
    const fn const_eq(self, other: Self) -> bool {
        self.0.const_eq(other.0)
    }

    /// equivalent to [`char::eq_ignore_ascii_case`] for `Utf8Char`
    #[must_use]
    pub const fn eq_ignore_ascii_case(self, other: Self) -> bool {
        self.to_ascii_lowercase()
            .const_eq(other.to_ascii_lowercase())
    }

    project_ascii!(
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
        is_ascii_whitespace,
    );

    /// equivalent to [`char::make_ascii_lowercase`] for `Utf8Char`
    pub const fn make_ascii_lowercase(&mut self) {
        *self = self.to_ascii_lowercase();
    }
    /// equivalent to [`char::make_ascii_uppercase`] for `Utf8Char`
    pub const fn make_ascii_uppercase(&mut self) {
        *self = self.to_ascii_uppercase();
    }
    /// equivalent to [`char::to_ascii_lowercase`] for `Utf8Char`
    #[must_use]
    pub const fn to_ascii_lowercase(mut self) -> Self {
        if self.is_ascii_uppercase() {
            // SAFETY: we only modify if is_ascii_uppercase is true (len: 1), taking it to another
            // valid ascii value (len: 1)
            let ascii = unsafe { self.0.first_byte_mut() };

            *ascii += b'a' - b'A';
        }

        self
    }
    /// equivalent to [`char::to_ascii_uppercase`] for `Utf8Char`
    #[must_use]
    pub const fn to_ascii_uppercase(mut self) -> Self {
        if self.is_ascii_lowercase() {
            // SAFETY: we only modify if is_ascii_lowercase is true (len: 1), taking it to another
            // valid ascii value (len: 1)
            let ascii = unsafe { self.0.first_byte_mut() };

            *ascii -= b'a' - b'A';
        }

        self
    }

    /// equivalent to [`char::is_digit`] for `Utf8Char`
    #[must_use]
    pub const fn is_digit(self, radix: u8) -> bool {
        self.to_digit(radix).is_some()
    }
    /// equivalent to [`char::to_digit`] for `Utf8Char`
    #[must_use]
    pub const fn to_digit(self, radix: u8) -> Option<u8> {
        // Copied completely from char::to_digit with slight tweaks to support a u8 based api

        // wraps on out of bounds characters
        let mut digit = self.ascii().wrapping_sub(b'0');

        if radix > 10 {
            assert!(
                radix >= 2 && radix <= 36,
                "to_digit: invalid radix -- radix must be in the range 2 to 36 inclusive"
            );
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

        for radix in 2..=36 {
            assert_eq!(utf8.is_digit(radix), c.is_digit(radix as u32));
            assert_eq!(
                utf8.to_digit(radix),
                c.to_digit(radix as u32).map(|n| n as u8)
            );
        }
    })
}
