//! Things copied from the standard library
//! Ideally this file becomes empty

use super::Utf8Char;

/// Const modified copy of `char::encode_utf8`
/// see inner comments for specific files and details
pub(crate) const fn from_char(code: char) -> Utf8Char {
    // this method is provably correct for all unicode characters: it is tested
    // this entire function is a const modified copy of the implementation of char::encode_utf8 in
    // core/char/methods.rs
    // FIXME(ultrabear): replace with encode_utf8 when const mut refs are stable (and
    // encode_utf8 is const stable)
    // FIXME(1.83): const_mut_refs and encode_utf8 const stable as of 1.83

    /// Continuation byte tag
    const TAG_CONTINUATION: u8 = 0b1000_0000;
    /// Tag to represent 2 byte codepoint
    const TAG_TWO: u8 = 0b1100_0000;
    /// Tag to represent 3 byte codepoint
    const TAG_THREE: u8 = 0b1110_0000;
    /// Tag to represent 4 byte codepoint
    const TAG_FOUR: u8 = 0b1111_0000;

    let mut out = [0xff; 4];

    let len = code.len_utf8();

    let code = code as u32;

    match len {
        1 => out[0] = truncate_u8(code),
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
    Utf8Char(out)
}

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

    let x = code.0[0];

    if x < 128 {
        return x as char;
    }

    let init = utf8_first_byte(x, 2);

    let y = code.0[1];

    let mut ch = utf8_acc_cont_byte(init, y);

    if x >= 0xE0 {
        let z = code.0[2];

        let y_z = utf8_acc_cont_byte((y & B6) as u32, z);

        ch = init << 12 | y_z;

        if x >= 0xF0 {
            let w = code.0[3];
            ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
        }
    }

    debug_assert!(char::from_u32(ch).is_some());

    // SAFETY: Utf8Char must always be valid utf8 so this must always be valid
    unsafe { char::from_u32_unchecked(ch) }
}

/// Truncates a u32 to a u8, exists for clippy compliance until const traits let us use
/// [`explicit_cast`](https://docs.rs/explicit_cast)
pub(crate) const fn truncate_u8(v: u32) -> u8 {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "this functions explicit definition is to truncate"
    )]
    let v = v as u8;

    v
}
