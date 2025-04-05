//! Things copied from the standard library
//! Ideally this file becomes empty

use super::Utf8Char;

/// Continuation byte tag
pub(crate) const TAG_CONTINUATION: u8 = 0b10_00_0000;

/// Const modified copy of `str::chars().next().unwrap_unchecked()`
/// see inner comments for details
pub(crate) const fn to_char(code: Utf8Char) -> char {
    // this method is provably correct for all unicode characters: it is tested below
    // this entire function is copied off of the implementation of str::chars() because it is
    // highly performant
    // the only relevant modifications are ones such that it may be fully used in a const
    // context
    // FIXME(ultrabear): replace this entire thing with chars().next().unwrap_unchecked() when
    // that is const stable

    // the below is mostly copied from core/src/str/validations.rs
    /// mask of continuation bytes data portion
    const B6: u8 = 0b0011_1111;

    /// reads first byte of utf8 data
    const fn utf8_first_byte(byte: u8, width: u32) -> u32 {
        (byte & (0x7F >> width)) as u32
    }

    /// adds a continuation byte to the char
    const fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
        (ch << 6) | (byte & B6) as u32
    }

    let arr = code.0.as_array();

    let x = arr[0];

    if x < 128 {
        return x as char;
    }

    let init = utf8_first_byte(x, 2);

    let y = arr[1];

    let mut ch = utf8_acc_cont_byte(init, y);

    if x >= 0xE0 {
        let z = arr[2];

        let y_z = utf8_acc_cont_byte((y & B6) as u32, z);

        ch = init << 12 | y_z;

        if x >= 0xF0 {
            let w = arr[3];
            ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
        }
    }

    debug_assert!(char::from_u32(ch).is_some());

    // SAFETY: Utf8Char must always be valid utf8 so this must always be valid
    unsafe { char::from_u32_unchecked(ch) }
}
