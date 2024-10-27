# utf8char
A char encoded as UTF-8, this has multiple advantages over the char primitive and a one codepoint &str.

Encoding and decoding char\<\-\>utf8 is expensive, yet common in cases where 
the advantages of the char data representation may not matter (see: `str::chars() -> transform -> collect::<String>()`).

