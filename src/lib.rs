//! A UTF-8 encoded codepoint.

#![no_std]
#![warn(clippy::pedantic)]
#![warn(missing_docs, clippy::missing_docs_in_private_items)]

use core::{
    borrow::Borrow,
    char,
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    hint::assert_unchecked as assume,
    ops::{Deref, DerefMut},
};

mod std_at_home;

/// A single unicode codepoint encoded in utf8.
///
/// This type is like a contemporary `char`:
/// - Debug/Display has identical output
/// - Represents a single unicode codepoint
/// - Has a size of 4 bytes
/// - Is Copy
///
/// However, being encoded as utf8;
/// - you can take a `&str` reference to it
/// - you can take a `&mut str` reference to it
/// - it is `Borrow<str>` and `PartialOrd<str>`
/// - it Hashes like `&str`
/// - it Ord's like `&str`
/// 
/// Encoding between a char and utf8 is expensive and branched, and clunky when you have a method
/// expecting a `&str`. Storing your data as a `&str` instead takes up 16 additional bytes even when 
/// its not needed in a `&str` representation. `Utf8Char` exists to fill this gap; "I have one codepoint 
/// but I want to use it with `&str` taking APIs". It is as compact as and holds the same guarantees as a 
/// `char`, but has the convenience and performance of a `&str`.
///
/// Not all char methods are provided, for most this is because their implementation 
/// would look like `self.to_char().method()`, causing an unexpected net negative in 
/// performance
#[derive(Copy, Clone)]
pub struct Utf8Char([u8; 4]);

impl Utf8Char {
    /// Returns the length of a UTF-8 encoded codepoint based on the first bytes
    /// encoding (returns 1..=4).
    ///
    /// `byte` must be the first byte of a valid UTF-8 encoded codepoint.
    #[must_use]
    const fn codepoint_len(byte: u8) -> u8 {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "leading_ones returns a u32 for some reason, this will only return 1..=4 on valid utf8"
        )]
        let len = (byte.leading_ones().saturating_sub(1) + 1) as u8;
        len
    }

    /// Returns the amount of bytes this codepoint takes up when encoded as utf8
    #[must_use]
    pub const fn len_utf8(&self) -> u8 {
        let len = Self::codepoint_len(self.0[0]);

        // SAFETY: codepoint_len will always return 1..=4 for valid utf8, which Utf8Char is
        // always assumed to be
        unsafe { assume(1 <= len && len <= 4) };

        len
    }

    /// returns a `Utf8Char` from the first char of a passed `&str`. Returns None if the string
    /// contained no characters (was empty)
    ///
    /// This can be more performant than calling [`from_char`][Utf8Char::from_char], as no complex utf8 conversion will be
    /// performed
    #[must_use]
    pub const fn from_first_char(s: &str) -> Option<Self> {
        // this method is provably correct for all unicode characters: it is tested below
        // writing it this way forces us to branch if the condition is not met
        let false = s.is_empty() else {
            return None;
        };

        // SAFETY: We have just checked the string is not empty, and returned otherwise
        Some(unsafe { Self::from_first_char_unchecked(s) })
    }

    /// returns a `Utf8Char` from the first char of a passed non empty string.
    ///
    /// # Safety
    /// This function must be called with a string that has a length greater than or equal to one.
    #[must_use]
    pub const unsafe fn from_first_char_unchecked(s: &str) -> Self {
        // SAFETY: the caller must always pass a nonempty string as a safety invariant
        unsafe { assume(!s.is_empty()) };

        let b = s.as_bytes();

        // expect this to not generate panic code thanks to our assume
        // cant use get_unchecked because it is not const
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
    #[must_use]
    pub const fn from_char(code: char) -> Self {
        std_at_home::from_char(code)
    }

    /// Converts a `Utf8Char` to a `char`
    #[must_use]
    pub const fn to_char(self) -> char {
        std_at_home::to_char(self)
    }

    /// Returns a string reference to the codepoint
    #[must_use]
    pub const fn as_str(&self) -> &str {
        let len = self.len_utf8() as usize;

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
        let len = self.len_utf8() as usize;

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
            f.write_str(self.as_str())
        } else {
            f.pad(self.as_str())
        }
    }
}

#[cfg(test)]
extern crate alloc;

#[test]
fn roundtrip() {
    use rayon::iter::{IntoParallelIterator, ParallelIterator};

    ('\0'..=char::MAX).into_par_iter().for_each(|ch| {
        let mut buf = [0xff; 4];

        let s = ch.encode_utf8(&mut buf);

        let utf8_alt = Utf8Char::from_first_char(s).unwrap();

        let utf8 = Utf8Char::from_char(ch);

        let codelen = Utf8Char::codepoint_len(utf8.0[0]);

        assert!(matches!(codelen, 1..=4));

        assert_eq!(codelen as usize, ch.len_utf8());
        assert_eq!(utf8.len_utf8() as usize, ch.len_utf8());

        assert_eq!(s, &*utf8);
        assert_eq!(&*utf8_alt, s);
        assert_eq!(buf, utf8.0);

        assert_eq!(utf8.to_char(), ch);
        assert_eq!(utf8_alt.to_char(), ch);
    })
}

#[test]
fn empty_string() {
    assert!(Utf8Char::from_first_char("").is_none());
}

#[test]
fn displays() {
    use alloc::string::String;
    use core::{fmt::Write, write};

    use rayon::iter::{IntoParallelIterator, ParallelIterator};

    ('\0'..=char::MAX).into_par_iter().for_each_with(
        (String::new(), String::new()),
        |(bufutf8, bufutf32), utf32| {
            let mut utf8 = Utf8Char::from_char(utf32);

            // Display
            bufutf8.clear();
            bufutf32.clear();

            write!(bufutf8, "{utf8}").unwrap();
            write!(bufutf32, "{utf32}").unwrap();

            assert_eq!(bufutf8, bufutf32);
            assert_eq!(bufutf8, utf8.as_str());
            assert_eq!(bufutf8, &*utf8.as_mut_str());

            assert_eq!(Utf8Char::from_first_char(&bufutf8), Some(utf8));
            assert_eq!(
                unsafe { Utf8Char::from_first_char_unchecked(&bufutf8) },
                utf8
            );

            // Debug
            bufutf8.clear();
            bufutf32.clear();

            write!(bufutf8, "{utf8:?}").unwrap();
            write!(bufutf32, "{utf32:?}").unwrap();

            assert_eq!(bufutf8, bufutf32);
        },
    );
}
