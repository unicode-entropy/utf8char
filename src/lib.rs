//! A UTF-8 encoded [`prim@char`].
//!
//! This alternate representation of a `char` allows for certain benefits:
//! - Deref to `&str`
//! - Hash/Eq/Ord like `&str`
//! - Smaller than `&'static str` for single codepoint cases (full const support included)
//! - Some `char` methods are implementable without the performance loss of `&str` -> `char` -> `&str` (for
//!   instance in a `str::chars()` loop)
//!
//! To get started, create a [`Utf8Char`].

#![no_std]
#![warn(clippy::pedantic)]
#![warn(missing_docs, clippy::missing_docs_in_private_items)]

use core::{
    borrow::Borrow,
    cmp::Ordering,
    hash::{Hash, Hasher},
    hint::assert_unchecked as assume,
    ops::Deref,
};

use representation::{codepoint_len_lut, Utf8CharInner};
use std_at_home::TAG_CONTINUATION;

mod charapi;
pub mod iter;
mod representation;
mod std_at_home;
#[cfg(test)]
mod tests;

pub use iter::{Utf8CharIter, IntoUtf8Chars};


/// A single unicode codepoint encoded in utf8.
///
/// This type is like a contemporary [`prim@char`]:
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
/// Encoding between a `char` and utf8 is expensive and branched, and clunky when you have a method
/// expecting a `&str`. Storing your data as a `&str` instead takes up 16 additional bytes even when
/// its not needed in a `&str` representation. `Utf8Char` exists to fill this gap; "I have one codepoint
/// but I want to use it with `&str` taking APIs". It is as compact as and holds the same guarantees as a
/// `char`, but has the convenience and performance of a `&str`.
///
/// Not all `char` methods are provided, for most this is because their implementation
/// would look like `self.to_char().method()`, causing an unexpected net negative in
/// performance
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Utf8Char(Utf8CharInner);

impl Utf8Char {
    /// Returns the length of a UTF-8 encoded codepoint based on the first bytes
    /// encoding (returns 1..=4).
    ///
    /// `byte` must be the first byte of a valid UTF-8 encoded codepoint.
    #[must_use]
    const fn codepoint_len(byte: u8) -> u8 {
        codepoint_len_lut(byte)
    }

    /// Returns the amount of bytes this codepoint takes up when encoded as utf8
    #[must_use]
    pub const fn len_utf8(self) -> u8 {
        self.0.len_utf8() as u8
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
        // forwards to assumes below: FIXME
        unsafe { assume(b.len() >= len as usize) };

        #[expect(
            clippy::items_after_statements,
            reason = "its placed here because its only relevant here"
        )]
        const PAD: u8 = TAG_CONTINUATION;

        let mut out = [b[0], PAD, PAD, PAD];

        if len > 1 {
            // SAFETY: FIXME compiler introduces panic branches despite above assume
            unsafe { assume(b.len() > 1) }
            out[1] = b[1];
        }

        if len > 2 {
            // SAFETY: FIXME compiler introduces panic branches despite above assume
            unsafe { assume(b.len() > 2) }
            out[2] = b[2];
        }

        if len > 3 {
            // SAFETY: FIXME compiler introduces panic branches despite above assume
            unsafe { assume(b.len() > 3) }
            out[3] = b[3];
        }

        // NOTE: We are a safety invariant, the [u8; 4] in Utf8Char must be valid utf8
        // SAFETY: we follow the valid utf8char representation; utf8 bytes followed by TAG_CONTINUATION
        Self(unsafe { Utf8CharInner::from_utf8char_array(out) })
    }

    /// Creates a `Utf8Char` from a `char`
    #[must_use]
    pub const fn from_char(code: char) -> Self {
        // uses TAG_CONTINUATION as padding (a logical invariant)

        let mut buf = [TAG_CONTINUATION; 4];

        code.encode_utf8(&mut buf);

        // SAFETY: we follow the valid utf8char representation; utf8 bytes followed by TAG_CONTINUATION
        Self(unsafe { Utf8CharInner::from_utf8char_array(buf) })
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
        let slice = unsafe { self.0.as_array().split_at_unchecked(len).0 };

        // SAFETY: [u8; byte_len] of Utf8Char must be valid Utf8
        unsafe { core::str::from_utf8_unchecked(slice) }
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

impl Deref for Utf8Char {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
extern crate alloc;

#[test]
fn roundtrip() {
    use rayon::iter::ParallelIterator;

    tests::all_chars().for_each(|ch| {
        let mut buf = [TAG_CONTINUATION; 4];

        let s = ch.encode_utf8(&mut buf);

        let utf8_alt = Utf8Char::from_first_char(s).unwrap();

        let utf8 = Utf8Char::from_char(ch);

        let codelen = Utf8Char::codepoint_len(utf8.0.first_byte().0 as u8);

        assert!(matches!(codelen, 1..=4));

        assert_eq!(codelen as usize, ch.len_utf8());
        assert_eq!(utf8.len_utf8() as usize, ch.len_utf8());

        assert_eq!(s, &*utf8);
        assert_eq!(&*utf8_alt, s);
        // ensure internal repr is consistent
        assert_eq!(&buf, utf8.0.as_array());
        assert_eq!(&buf, utf8_alt.0.as_array());

        assert_eq!(utf8.to_char(), ch);
        assert_eq!(utf8_alt.to_char(), ch);
    });
}

#[test]
fn empty_string() {
    assert!(Utf8Char::from_first_char("").is_none());
}

#[test]
fn displays() {
    use alloc::string::String;
    use core::{fmt::Write, write};

    let test_char = |(bufutf8, bufutf32): &mut (String, String), utf32| {
        let utf8 = Utf8Char::from_char(utf32);

        // Display
        bufutf8.clear();
        bufutf32.clear();

        write!(bufutf8, "{utf8}").unwrap();
        write!(bufutf32, "{utf32}").unwrap();

        assert_eq!(bufutf8, bufutf32);
        assert_eq!(bufutf8, utf8.as_str());

        assert_eq!(Utf8Char::from_first_char(bufutf8), Some(utf8));
        assert_eq!(
            unsafe { Utf8Char::from_first_char_unchecked(bufutf8) },
            utf8
        );

        // Debug
        bufutf8.clear();
        bufutf32.clear();

        write!(bufutf8, "{utf8:?}").unwrap();
        write!(bufutf32, "{utf32:?}").unwrap();

        assert_eq!(bufutf8, bufutf32);
    };

    #[cfg(not(miri))]
    {
        use rayon::iter::ParallelIterator;
        tests::all_chars().for_each_with((String::new(), String::new()), test_char);
    }

    #[cfg(miri)]
    {
        let mut bufs = (String::new(), String::new());

        tests::all_chars().for_each(|ch| test_char(&mut bufs, ch));
    }
}

#[test]
fn total_ordering() {
    use rayon::iter::ParallelIterator;

    tests::all_char_pairs().for_each(|(a, b)| {
        let a_utf8 = Utf8Char::from_char(a);
        let b_utf8 = Utf8Char::from_char(b);

        assert_eq!(a_utf8.cmp(&b_utf8), a.cmp(&b));
        assert_eq!(
            a_utf8.eq_ignore_ascii_case(b_utf8),
            a.eq_ignore_ascii_case(&b)
        );
    });
}
