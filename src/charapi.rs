//! Containing file for API's that mimic `char` behaviour

use core::fmt;

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
// skip if the implementation would fallback to to_char().method()
#[allow(unused)]
impl Utf8Char {
    const fn eq_ignore_ascii_case(self, other: Self) -> bool {
        todo!()
    }

    fn make_ascii_lowercase(&mut self) {
        todo!()
    }

    const fn to_ascii_lowercase(self) -> Self {
        todo!()
    }

    const fn is_ascii(self) -> bool {
        todo!()
    }

    const fn is_ascii_alphabetic(self) -> bool {
        todo!()
    }




}
