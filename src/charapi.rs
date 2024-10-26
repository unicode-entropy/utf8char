//! Containing file for API's that mimic `char` behaviour 

use core::{
    borrow::Borrow,
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
};

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

impl Utf8Char {

    pub const fn eq_ignore_ascii_case(self, other: Utf8Char) -> bool {
        false
    }

}
