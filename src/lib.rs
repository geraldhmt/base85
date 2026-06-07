//! A library for Base85 encoding as described in [RFC1924](https://datatracker.ietf.org/doc/html/rfc1924) and released under the Mozilla Public License 2.0.
//!
//!## Description
//!
//! Several variants of Base85 encoding exist. The most popular variant is often known as ascii85 and is best known for use in Adobe products. This is not that algorithm.
//!
//! The variant implemented in RFC 1924 was originally intended for encoding IPv6 addresses. It utilizes the same concepts as other versions, but uses a character set which is friendly toward embedding in source code without the need for escaping. During decoding ASCII whitespace (\n, \r, \t, space) is ignored. A base85-encoded string is 25% larger than the original binary data, which is more efficient than the more-common base64 algorithm (33%). This encoding pairs very well with JSON, yielding lower overhead and needing no character escapes.
//!
//! ## Usage
//!
//! This was my first real Rust project but has matured since then and is stable. The API is simple: `encode()` turns a slice of bytes into a String and `decode()` turns a string reference into a Vector of bytes (u8). Both calls work completely within RAM, so processing huge files is probably not a good idea.
//!
//! ## Contributions
//!
//! Even though I've been coding for a while and have learned quite a bit about Rust, I'm still a novice. Suggestions and contributions are always welcome and appreciated.

use core::mem::MaybeUninit;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unexpected end of input")]
    UnexpectedEof,
    #[error("Unexpected character '{0}'")]
    InvalidCharacter(u8),
    #[error("Group is out of range for valid Base85 (exceeds u32::MAX)")]
    Overflow,
}

#[inline]
fn byte_to_char85(x85: u8) -> u8 {
    static B85_TO_CHAR: &[u8] =
        b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&()*+-;<=>?@^_`{|}~";
    B85_TO_CHAR[x85 as usize]
}

#[inline]
fn char85_to_byte(c: u8) -> Result<u8> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'A'..=b'Z' => Ok(c - b'A' + 10),
        b'a'..=b'z' => Ok(c - b'a' + 36),
        b'!' => Ok(62),
        b'#' => Ok(63),
        b'$' => Ok(64),
        b'%' => Ok(65),
        b'&' => Ok(66),
        b'(' => Ok(67),
        b')' => Ok(68),
        b'*' => Ok(69),
        b'+' => Ok(70),
        b'-' => Ok(71),
        b';' => Ok(72),
        b'<' => Ok(73),
        b'=' => Ok(74),
        b'>' => Ok(75),
        b'?' => Ok(76),
        b'@' => Ok(77),
        b'^' => Ok(78),
        b'_' => Ok(79),
        b'`' => Ok(80),
        b'{' => Ok(81),
        b'|' => Ok(82),
        b'}' => Ok(83),
        b'~' => Ok(84),
        v => Err(Error::InvalidCharacter(v)),
    }
}

/// encode() turns a slice of bytes into a string of encoded data
pub fn encode(indata: &[u8]) -> String {
    let chunks = indata.chunks_exact(4);
    let remainder = chunks.remainder();
    let capacity = if remainder.is_empty() {
        (indata.len() / 4) * 5
    } else {
        (indata.len() / 4) * 5 + remainder.len() + 1
    };
    let mut out = Vec::<MaybeUninit<u8>>::with_capacity(capacity);
    unsafe {
        out.set_len(capacity);
    }
    let mut out_chunks = out.chunks_exact_mut(5);

    for (chunk, out) in std::iter::zip(chunks, &mut out_chunks) {
        let decnum = u32::from_be_bytes(<[u8; 4]>::try_from(chunk).unwrap());
        out[0] = MaybeUninit::new(byte_to_char85((decnum / 85u32.pow(4)) as u8));
        out[1] = MaybeUninit::new(byte_to_char85(
            ((decnum % 85u32.pow(4)) / 85u32.pow(3)) as u8,
        ));
        out[2] = MaybeUninit::new(byte_to_char85(
            ((decnum % 85u32.pow(3)) / 85u32.pow(2)) as u8,
        ));
        out[3] = MaybeUninit::new(byte_to_char85(((decnum % 85u32.pow(2)) / 85u32) as u8));
        out[4] = MaybeUninit::new(byte_to_char85((decnum % 85u32) as u8));
    }

    let out_remainder = out_chunks.into_remainder();
    if let Some(a) = remainder.first().copied() {
        let b = remainder.get(1).copied();
        let c = remainder.get(2).copied();
        let d = remainder.get(3).copied();
        let decnum = u32::from_be_bytes([a, b.unwrap_or(0), c.unwrap_or(0), d.unwrap_or(0)]);
        out_remainder[0] = MaybeUninit::new(byte_to_char85((decnum / 85u32.pow(4)) as u8));
        out_remainder[1] = MaybeUninit::new(byte_to_char85(
            ((decnum % 85u32.pow(4)) / 85u32.pow(3)) as u8,
        ));
        if b.is_some() {
            out_remainder[2] = MaybeUninit::new(byte_to_char85(
                ((decnum % 85u32.pow(3)) / 85u32.pow(2)) as u8,
            ));
        }
        if c.is_some() {
            out_remainder[3] =
                MaybeUninit::new(byte_to_char85(((decnum % 85u32.pow(2)) / 85u32) as u8));
        }
        if d.is_some() {
            out_remainder[4] = MaybeUninit::new(byte_to_char85((decnum % 85u32) as u8));
        }
    }

    unsafe { String::from_utf8_unchecked(std::mem::transmute::<_, Vec<u8>>(out)) }
}

/// decode() turns a string of encoded data into a slice of bytes
pub fn decode(instr: &str) -> Result<Vec<u8>> {
    let indata = instr.as_bytes();
    let chunks = indata.chunks_exact(5);
    let remainder = chunks.remainder();
    let capacity = if remainder.is_empty() {
        (indata.len() / 5) * 4
    } else {
        (indata.len() / 5) * 4 + remainder.len() - 1
    };
    let mut out = Vec::<MaybeUninit<u8>>::with_capacity(capacity);
    unsafe {
        out.set_len(capacity);
    }
    let mut out_chunks = out.chunks_exact_mut(4);

    for (chunk, out_chunk) in std::iter::zip(chunks, &mut out_chunks) {
        // A 5-character group can reach 84 * (85^4 + ... + 1) = 4_437_053_124, past
        // u32::MAX, so accumulate in u64 and reject groups that don't fit in four bytes.
        let accumulator = u64::from(char85_to_byte(chunk[0])?) * 85u64.pow(4)
            + u64::from(char85_to_byte(chunk[1])?) * 85u64.pow(3)
            + u64::from(char85_to_byte(chunk[2])?) * 85u64.pow(2)
            + u64::from(char85_to_byte(chunk[3])?) * 85u64
            + u64::from(char85_to_byte(chunk[4])?);
        if accumulator > u64::from(u32::MAX) {
            return Err(Error::Overflow);
        }
        let accumulator = accumulator as u32;
        out_chunk[0] = MaybeUninit::new((accumulator >> 24) as u8);
        out_chunk[1] = MaybeUninit::new((accumulator >> 16) as u8);
        out_chunk[2] = MaybeUninit::new((accumulator >> 8) as u8);
        out_chunk[3] = MaybeUninit::new(accumulator as u8);
    }

    let out_remainder = out_chunks.into_remainder();
    if let Some(a) = remainder.first().copied() {
        let b = remainder.get(1).copied();
        let c = remainder.get(2).copied();
        let d = remainder.get(3).copied();
        let e = remainder.get(4).copied();
        let accumulator = u64::from(char85_to_byte(a)?) * 85u64.pow(4)
            + u64::from(b.map_or(Err(Error::UnexpectedEof), char85_to_byte)?) * 85u64.pow(3)
            + u64::from(c.map_or(Ok(126), char85_to_byte)?) * 85u64.pow(2)
            + u64::from(d.map_or(Ok(126), char85_to_byte)?) * 85u64.pow(1)
            + u64::from(e.map_or(Ok(126), char85_to_byte)?) * 85u64.pow(0);
        if accumulator > u64::from(u32::MAX) {
            return Err(Error::Overflow);
        }
        let accumulator = accumulator as u32;
        out_remainder[0] = MaybeUninit::new((accumulator >> 24) as u8);
        if remainder.len() > 2 {
            out_remainder[1] = MaybeUninit::new((accumulator >> 16) as u8);
            if remainder.len() > 3 {
                out_remainder[2] = MaybeUninit::new((accumulator >> 8) as u8);
                if remainder.len() > 4 {
                    out_remainder[3] = MaybeUninit::new(accumulator as u8);
                }
            }
        }
    }

    Ok(unsafe { std::mem::transmute::<_, Vec<u8>>(out) })
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_encode_decode() {
        // The list of tests consists of the unencoded data on the left and the encoded data on
        // the right. By using strings for the arbitrary binary data, we make the test much less
        // complicated to write.
        let testlist = [
            ("a", "VE"),
            ("aa", "VPO"),
            ("aaa", "VPRn"),
            ("aaaa", "VPRom"),
            ("aaaaa", "VPRomVE"),
            ("aaaaaa", "VPRomVPO"),
            ("aaaaaaa", "VPRomVPRn"),
            ("aaaaaaaa", "VPRomVPRom"),
        ];

        for test in testlist.iter() {
            let s = encode(test.0.as_bytes());
            assert_eq!(
                s, test.1,
                "encoder test failed: wanted: {}, got: {}",
                test.0, s
            );

            let b = decode(test.1)
                .unwrap_or_else(|e| panic!("decoder test error on input {}: {}", test.1, e));

            let s = String::from_utf8(b).unwrap_or_else(|e| {
                panic!(
                    "decoder test '{}' failed to convert to string: {:#?}",
                    test.1, e
                )
            });

            assert_eq!(
                test.0, s,
                "decoder data mismatch: wanted: {}, got: {}",
                test.0, s
            );
        }
    }

    #[test]
    fn test_overflow_groups_return_error() {
        // Five '~' (84 each) accumulate to 4_437_053_124, past u32::MAX: this panicked
        // in debug and produced wrong bytes in release before the fix.
        assert!(matches!(decode("~~~~~"), Err(Error::Overflow)));

        // Same overflow in the trailing-remainder block: "~~" reaches 4_384_852_500.
        assert!(matches!(decode("~~"), Err(Error::Overflow)));

        // A lone character is an incomplete group: it must still return UnexpectedEof
        // rather than panicking on the overflowing multiply first.
        assert!(matches!(decode("}"), Err(Error::UnexpectedEof)));

        // Values right up to the u32 ceiling must still round-trip unchanged.
        let max = encode(&[0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(decode(&max).unwrap(), vec![0xFF, 0xFF, 0xFF, 0xFF]);
        let one = encode(&[0xFF]);
        assert_eq!(decode(&one).unwrap(), vec![0xFF]);
    }
}
