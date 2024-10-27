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
// skip if the implementation would be faster as to_char().method()
#[allow(unused)]
impl Utf8Char {

    const fn ascii(self) -> u8 {
        self.0.first_byte()
    }

    const fn const_eq(self, other: Self )-> bool {
        self.0.const_eq(other.0)
    }


    const fn eq_ignore_ascii_case(self, other: Self) -> bool { self.to_ascii_lowercase().const_eq(other.to_ascii_lowercase()) }

    const fn is_ascii(self) -> bool { matches!(self.ascii(), 0..=127) }
    const fn is_ascii_alphabetic(self) -> bool { matches!(self.ascii(), b'a'..=b'z' | b'A'..=b'Z') }
    const fn is_ascii_alphanumeric(self) -> bool { self.is_ascii_alphabetic() | self.is_ascii_digit() }
    const fn is_ascii_control(&self) -> bool { 
        // copied from std char impl; I have no clue what counts 
        matches!(self.ascii(), b'\0'..=b'\x1F' | b'\x7F' ) }
    const fn is_ascii_digit(&self) -> bool { matches!(self.ascii(), b'0'..=b'9') }
    const fn is_ascii_graphic(&self) -> bool { matches!(self.ascii(), b'!'..=b'~') }
    const fn is_ascii_hexdigit(&self) -> bool { todo!() }
    const fn is_ascii_lowercase(&self) -> bool { todo!() }
    const fn is_ascii_punctuation(&self) -> bool { todo!() }
    const fn is_ascii_uppercase(&self) -> bool { todo!() }
    const fn is_ascii_whitespace(&self) -> bool { todo!() }


    fn make_ascii_lowercase(&mut self) { todo!() }
    fn make_ascii_uppercase(&mut self) { todo!() }
    const fn to_ascii_lowercase(self) -> Self { todo!() }
    const fn to_ascii_uppercase(self) -> Self { todo!() }

    const fn is_digit(self, radix: u8) -> bool       { todo!() }
    const fn to_digit(self, radix: u8) -> Option<u8> { todo!() }
}
