//! Enums that are used by the internal representation of `Utf8Char`

/// Enum representing every valid state of the first byte of a utf8 encoded codepoint
#[repr(u8)]
// The ascii variants were copied over from the rust standard library
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
#[expect(
    dead_code,
    reason = "rust cant see when we transmute into/outof these values"
)]
pub(crate) enum Utf8FirstByte {
    /// U+0000 (The default variant)
    Null = 0,
    /// U+0001
    StartOfHeading = 1,
    /// U+0002
    StartOfText = 2,
    /// U+0003
    EndOfText = 3,
    /// U+0004
    EndOfTransmission = 4,
    /// U+0005
    Enquiry = 5,
    /// U+0006
    Acknowledge = 6,
    /// U+0007
    Bell = 7,
    /// U+0008
    Backspace = 8,
    /// U+0009
    CharacterTabulation = 9,
    /// U+000A
    LineFeed = 10,
    /// U+000B
    LineTabulation = 11,
    /// U+000C
    FormFeed = 12,
    /// U+000D
    CarriageReturn = 13,
    /// U+000E
    ShiftOut = 14,
    /// U+000F
    ShiftIn = 15,
    /// U+0010
    DataLinkEscape = 16,
    /// U+0011
    DeviceControlOne = 17,
    /// U+0012
    DeviceControlTwo = 18,
    /// U+0013
    DeviceControlThree = 19,
    /// U+0014
    DeviceControlFour = 20,
    /// U+0015
    NegativeAcknowledge = 21,
    /// U+0016
    SynchronousIdle = 22,
    /// U+0017
    EndOfTransmissionBlock = 23,
    /// U+0018
    Cancel = 24,
    /// U+0019
    EndOfMedium = 25,
    /// U+001A
    Substitute = 26,
    /// U+001B
    Escape = 27,
    /// U+001C
    InformationSeparatorFour = 28,
    /// U+001D
    InformationSeparatorThree = 29,
    /// U+001E
    InformationSeparatorTwo = 30,
    /// U+001F
    InformationSeparatorOne = 31,
    /// U+0020
    Space = 32,
    /// U+0021
    ExclamationMark = 33,
    /// U+0022
    QuotationMark = 34,
    /// U+0023
    NumberSign = 35,
    /// U+0024
    DollarSign = 36,
    /// U+0025
    PercentSign = 37,
    /// U+0026
    Ampersand = 38,
    /// U+0027
    Apostrophe = 39,
    /// U+0028
    LeftParenthesis = 40,
    /// U+0029
    RightParenthesis = 41,
    /// U+002A
    Asterisk = 42,
    /// U+002B
    PlusSign = 43,
    /// U+002C
    Comma = 44,
    /// U+002D
    HyphenMinus = 45,
    /// U+002E
    FullStop = 46,
    /// U+002F
    Solidus = 47,
    /// U+0030
    Digit0 = 48,
    /// U+0031
    Digit1 = 49,
    /// U+0032
    Digit2 = 50,
    /// U+0033
    Digit3 = 51,
    /// U+0034
    Digit4 = 52,
    /// U+0035
    Digit5 = 53,
    /// U+0036
    Digit6 = 54,
    /// U+0037
    Digit7 = 55,
    /// U+0038
    Digit8 = 56,
    /// U+0039
    Digit9 = 57,
    /// U+003A
    Colon = 58,
    /// U+003B
    Semicolon = 59,
    /// U+003C
    LessThanSign = 60,
    /// U+003D
    EqualsSign = 61,
    /// U+003E
    GreaterThanSign = 62,
    /// U+003F
    QuestionMark = 63,
    /// U+0040
    CommercialAt = 64,
    /// U+0041
    CapitalA = 65,
    /// U+0042
    CapitalB = 66,
    /// U+0043
    CapitalC = 67,
    /// U+0044
    CapitalD = 68,
    /// U+0045
    CapitalE = 69,
    /// U+0046
    CapitalF = 70,
    /// U+0047
    CapitalG = 71,
    /// U+0048
    CapitalH = 72,
    /// U+0049
    CapitalI = 73,
    /// U+004A
    CapitalJ = 74,
    /// U+004B
    CapitalK = 75,
    /// U+004C
    CapitalL = 76,
    /// U+004D
    CapitalM = 77,
    /// U+004E
    CapitalN = 78,
    /// U+004F
    CapitalO = 79,
    /// U+0050
    CapitalP = 80,
    /// U+0051
    CapitalQ = 81,
    /// U+0052
    CapitalR = 82,
    /// U+0053
    CapitalS = 83,
    /// U+0054
    CapitalT = 84,
    /// U+0055
    CapitalU = 85,
    /// U+0056
    CapitalV = 86,
    /// U+0057
    CapitalW = 87,
    /// U+0058
    CapitalX = 88,
    /// U+0059
    CapitalY = 89,
    /// U+005A
    CapitalZ = 90,
    /// U+005B
    LeftSquareBracket = 91,
    /// U+005C
    ReverseSolidus = 92,
    /// U+005D
    RightSquareBracket = 93,
    /// U+005E
    CircumflexAccent = 94,
    /// U+005F
    LowLine = 95,
    /// U+0060
    GraveAccent = 96,
    /// U+0061
    SmallA = 97,
    /// U+0062
    SmallB = 98,
    /// U+0063
    SmallC = 99,
    /// U+0064
    SmallD = 100,
    /// U+0065
    SmallE = 101,
    /// U+0066
    SmallF = 102,
    /// U+0067
    SmallG = 103,
    /// U+0068
    SmallH = 104,
    /// U+0069
    SmallI = 105,
    /// U+006A
    SmallJ = 106,
    /// U+006B
    SmallK = 107,
    /// U+006C
    SmallL = 108,
    /// U+006D
    SmallM = 109,
    /// U+006E
    SmallN = 110,
    /// U+006F
    SmallO = 111,
    /// U+0070
    SmallP = 112,
    /// U+0071
    SmallQ = 113,
    /// U+0072
    SmallR = 114,
    /// U+0073
    SmallS = 115,
    /// U+0074
    SmallT = 116,
    /// U+0075
    SmallU = 117,
    /// U+0076
    SmallV = 118,
    /// U+0077
    SmallW = 119,
    /// U+0078
    SmallX = 120,
    /// U+0079
    SmallY = 121,
    /// U+007A
    SmallZ = 122,
    /// U+007B
    LeftCurlyBracket = 123,
    /// U+007C
    VerticalLine = 124,
    /// U+007D
    RightCurlyBracket = 125,
    /// U+007E
    Tilde = 126,
    /// U+007F
    Delete = 127,

    // Begin outside of ascii range
    // Generated by python script utf8firstbyte.py

    // SKIP: _128 = 0b1000_0000, unicode cont bytes are invalid
    // SKIP: _129 = 0b1000_0001, unicode cont bytes are invalid
    // SKIP: _130 = 0b1000_0010, unicode cont bytes are invalid
    // SKIP: _131 = 0b1000_0011, unicode cont bytes are invalid
    // SKIP: _132 = 0b1000_0100, unicode cont bytes are invalid
    // SKIP: _133 = 0b1000_0101, unicode cont bytes are invalid
    // SKIP: _134 = 0b1000_0110, unicode cont bytes are invalid
    // SKIP: _135 = 0b1000_0111, unicode cont bytes are invalid
    // SKIP: _136 = 0b1000_1000, unicode cont bytes are invalid
    // SKIP: _137 = 0b1000_1001, unicode cont bytes are invalid
    // SKIP: _138 = 0b1000_1010, unicode cont bytes are invalid
    // SKIP: _139 = 0b1000_1011, unicode cont bytes are invalid
    // SKIP: _140 = 0b1000_1100, unicode cont bytes are invalid
    // SKIP: _141 = 0b1000_1101, unicode cont bytes are invalid
    // SKIP: _142 = 0b1000_1110, unicode cont bytes are invalid
    // SKIP: _143 = 0b1000_1111, unicode cont bytes are invalid
    // SKIP: _144 = 0b1001_0000, unicode cont bytes are invalid
    // SKIP: _145 = 0b1001_0001, unicode cont bytes are invalid
    // SKIP: _146 = 0b1001_0010, unicode cont bytes are invalid
    // SKIP: _147 = 0b1001_0011, unicode cont bytes are invalid
    // SKIP: _148 = 0b1001_0100, unicode cont bytes are invalid
    // SKIP: _149 = 0b1001_0101, unicode cont bytes are invalid
    // SKIP: _150 = 0b1001_0110, unicode cont bytes are invalid
    // SKIP: _151 = 0b1001_0111, unicode cont bytes are invalid
    // SKIP: _152 = 0b1001_1000, unicode cont bytes are invalid
    // SKIP: _153 = 0b1001_1001, unicode cont bytes are invalid
    // SKIP: _154 = 0b1001_1010, unicode cont bytes are invalid
    // SKIP: _155 = 0b1001_1011, unicode cont bytes are invalid
    // SKIP: _156 = 0b1001_1100, unicode cont bytes are invalid
    // SKIP: _157 = 0b1001_1101, unicode cont bytes are invalid
    // SKIP: _158 = 0b1001_1110, unicode cont bytes are invalid
    // SKIP: _159 = 0b1001_1111, unicode cont bytes are invalid
    // SKIP: _160 = 0b1010_0000, unicode cont bytes are invalid
    // SKIP: _161 = 0b1010_0001, unicode cont bytes are invalid
    // SKIP: _162 = 0b1010_0010, unicode cont bytes are invalid
    // SKIP: _163 = 0b1010_0011, unicode cont bytes are invalid
    // SKIP: _164 = 0b1010_0100, unicode cont bytes are invalid
    // SKIP: _165 = 0b1010_0101, unicode cont bytes are invalid
    // SKIP: _166 = 0b1010_0110, unicode cont bytes are invalid
    // SKIP: _167 = 0b1010_0111, unicode cont bytes are invalid
    // SKIP: _168 = 0b1010_1000, unicode cont bytes are invalid
    // SKIP: _169 = 0b1010_1001, unicode cont bytes are invalid
    // SKIP: _170 = 0b1010_1010, unicode cont bytes are invalid
    // SKIP: _171 = 0b1010_1011, unicode cont bytes are invalid
    // SKIP: _172 = 0b1010_1100, unicode cont bytes are invalid
    // SKIP: _173 = 0b1010_1101, unicode cont bytes are invalid
    // SKIP: _174 = 0b1010_1110, unicode cont bytes are invalid
    // SKIP: _175 = 0b1010_1111, unicode cont bytes are invalid
    // SKIP: _176 = 0b1011_0000, unicode cont bytes are invalid
    // SKIP: _177 = 0b1011_0001, unicode cont bytes are invalid
    // SKIP: _178 = 0b1011_0010, unicode cont bytes are invalid
    // SKIP: _179 = 0b1011_0011, unicode cont bytes are invalid
    // SKIP: _180 = 0b1011_0100, unicode cont bytes are invalid
    // SKIP: _181 = 0b1011_0101, unicode cont bytes are invalid
    // SKIP: _182 = 0b1011_0110, unicode cont bytes are invalid
    // SKIP: _183 = 0b1011_0111, unicode cont bytes are invalid
    // SKIP: _184 = 0b1011_1000, unicode cont bytes are invalid
    // SKIP: _185 = 0b1011_1001, unicode cont bytes are invalid
    // SKIP: _186 = 0b1011_1010, unicode cont bytes are invalid
    // SKIP: _187 = 0b1011_1011, unicode cont bytes are invalid
    // SKIP: _188 = 0b1011_1100, unicode cont bytes are invalid
    // SKIP: _189 = 0b1011_1101, unicode cont bytes are invalid
    // SKIP: _190 = 0b1011_1110, unicode cont bytes are invalid
    // SKIP: _191 = 0b1011_1111, unicode cont bytes are invalid
    // SKIP: _192 = 0b1100_0000, guaranteed overlong encodings are invalid
    // SKIP: _193 = 0b1100_0001, guaranteed overlong encodings are invalid
    _194 = 0b1100_0010, // c2
    _195 = 0b1100_0011, // c3
    _196 = 0b1100_0100, // c4
    _197 = 0b1100_0101, // c5
    _198 = 0b1100_0110, // c6
    _199 = 0b1100_0111, // c7
    _200 = 0b1100_1000, // c8
    _201 = 0b1100_1001, // c9
    _202 = 0b1100_1010, // ca
    _203 = 0b1100_1011, // cb
    _204 = 0b1100_1100, // cc
    _205 = 0b1100_1101, // cd
    _206 = 0b1100_1110, // ce
    _207 = 0b1100_1111, // cf
    _208 = 0b1101_0000, // d0
    _209 = 0b1101_0001, // d1
    _210 = 0b1101_0010, // d2
    _211 = 0b1101_0011, // d3
    _212 = 0b1101_0100, // d4
    _213 = 0b1101_0101, // d5
    _214 = 0b1101_0110, // d6
    _215 = 0b1101_0111, // d7
    _216 = 0b1101_1000, // d8
    _217 = 0b1101_1001, // d9
    _218 = 0b1101_1010, // da
    _219 = 0b1101_1011, // db
    _220 = 0b1101_1100, // dc
    _221 = 0b1101_1101, // dd
    _222 = 0b1101_1110, // de
    _223 = 0b1101_1111, // df
    _224 = 0b1110_0000, // e0
    _225 = 0b1110_0001, // e1
    _226 = 0b1110_0010, // e2
    _227 = 0b1110_0011, // e3
    _228 = 0b1110_0100, // e4
    _229 = 0b1110_0101, // e5
    _230 = 0b1110_0110, // e6
    _231 = 0b1110_0111, // e7
    _232 = 0b1110_1000, // e8
    _233 = 0b1110_1001, // e9
    _234 = 0b1110_1010, // ea
    _235 = 0b1110_1011, // eb
    _236 = 0b1110_1100, // ec
    _237 = 0b1110_1101, // ed
    _238 = 0b1110_1110, // ee
    _239 = 0b1110_1111, // ef
    _240 = 0b1111_0000, // f0
    _241 = 0b1111_0001, // f1
    _242 = 0b1111_0010, // f2
    _243 = 0b1111_0011, // f3
    _244 = 0b1111_0100, // f4
    _245 = 0b1111_0101, // f5
    _246 = 0b1111_0110, // f6
    _247 = 0b1111_0111, // f7
                        // SKIP: _248 = 0b1111_1000, everything with 5 contiguous 1 bits is invalid
                        // SKIP: _249 = 0b1111_1001, everything with 5 contiguous 1 bits is invalid
                        // SKIP: _250 = 0b1111_1010, everything with 5 contiguous 1 bits is invalid
                        // SKIP: _251 = 0b1111_1011, everything with 5 contiguous 1 bits is invalid
                        // SKIP: _252 = 0b1111_1100, everything with 5 contiguous 1 bits is invalid
                        // SKIP: _253 = 0b1111_1101, everything with 5 contiguous 1 bits is invalid
                        // SKIP: _254 = 0b1111_1110, everything with 5 contiguous 1 bits is invalid
                        // SKIP: _255 = 0b1111_1111, everything with 5 contiguous 1 bits is invalid
}

/// Enum representing every valid state of continuation bytes of a utf8 encoded codepoint
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u8)]
pub(crate) enum Utf8ContByte {
    // Generated by utf8contbyte.py
    _128 = 0b1000_0000, // 80
    _129 = 0b1000_0001, // 81
    _130 = 0b1000_0010, // 82
    _131 = 0b1000_0011, // 83
    _132 = 0b1000_0100, // 84
    _133 = 0b1000_0101, // 85
    _134 = 0b1000_0110, // 86
    _135 = 0b1000_0111, // 87
    _136 = 0b1000_1000, // 88
    _137 = 0b1000_1001, // 89
    _138 = 0b1000_1010, // 8a
    _139 = 0b1000_1011, // 8b
    _140 = 0b1000_1100, // 8c
    _141 = 0b1000_1101, // 8d
    _142 = 0b1000_1110, // 8e
    _143 = 0b1000_1111, // 8f
    _144 = 0b1001_0000, // 90
    _145 = 0b1001_0001, // 91
    _146 = 0b1001_0010, // 92
    _147 = 0b1001_0011, // 93
    _148 = 0b1001_0100, // 94
    _149 = 0b1001_0101, // 95
    _150 = 0b1001_0110, // 96
    _151 = 0b1001_0111, // 97
    _152 = 0b1001_1000, // 98
    _153 = 0b1001_1001, // 99
    _154 = 0b1001_1010, // 9a
    _155 = 0b1001_1011, // 9b
    _156 = 0b1001_1100, // 9c
    _157 = 0b1001_1101, // 9d
    _158 = 0b1001_1110, // 9e
    _159 = 0b1001_1111, // 9f
    _160 = 0b1010_0000, // a0
    _161 = 0b1010_0001, // a1
    _162 = 0b1010_0010, // a2
    _163 = 0b1010_0011, // a3
    _164 = 0b1010_0100, // a4
    _165 = 0b1010_0101, // a5
    _166 = 0b1010_0110, // a6
    _167 = 0b1010_0111, // a7
    _168 = 0b1010_1000, // a8
    _169 = 0b1010_1001, // a9
    _170 = 0b1010_1010, // aa
    _171 = 0b1010_1011, // ab
    _172 = 0b1010_1100, // ac
    _173 = 0b1010_1101, // ad
    _174 = 0b1010_1110, // ae
    _175 = 0b1010_1111, // af
    _176 = 0b1011_0000, // b0
    _177 = 0b1011_0001, // b1
    _178 = 0b1011_0010, // b2
    _179 = 0b1011_0011, // b3
    _180 = 0b1011_0100, // b4
    _181 = 0b1011_0101, // b5
    _182 = 0b1011_0110, // b6
    _183 = 0b1011_0111, // b7
    _184 = 0b1011_1000, // b8
    _185 = 0b1011_1001, // b9
    _186 = 0b1011_1010, // ba
    _187 = 0b1011_1011, // bb
    _188 = 0b1011_1100, // bc
    _189 = 0b1011_1101, // bd
    _190 = 0b1011_1110, // be
    _191 = 0b1011_1111, // bf
}
