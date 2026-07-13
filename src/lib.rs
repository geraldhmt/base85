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
//! ## performance / safe code only
//!
//! this crate is guide by performance. We do some optimization which required small unsafe section.
//!
//! If you absolue want to use only safe code, we add a feature : only_safe to allow you to chose a lite performance degradation but only safe code is used.
//!
//! We recommend to use bench (see readme) to compare performance in your use cases and choose.
//!
//! ## Contributions
//!
//! Even though I've been coding for a while and have learned quite a bit about Rust, I'm still a novice. Suggestions and contributions are always welcome and appreciated.

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unexpected end of input")]
    UnexpectedEof,
    #[error("Unexpected character '0x{0:02X}'")]
    InvalidCharacter(u8),
    #[error("Output buffer too small")]
    OutputBufferTooSmall,
}

// Ref : https://www.rfc-editor.org/rfc/rfc1924
const RFC1924_ALPHABET_LEN: usize = 85;
const RFC1924_ALPHABET: &[u8; RFC1924_ALPHABET_LEN] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&()*+-;<=>?@^_`{|}~";

// Do at compile time to avoid the overhead of building the table at runtime.
// No iterator available at compile time, so we have to generate the table with a const function.
const U8_COUNT_VALUE: usize = std::u8::MAX as usize + 1;
const fn generate_rfc1924_decode_table() -> [Option<u8>; U8_COUNT_VALUE] {
    let mut table: [Option<u8>; U8_COUNT_VALUE] = [None; U8_COUNT_VALUE];
    let mut i = 0;
    assert!(
        RFC1924_ALPHABET.len() == RFC1924_ALPHABET_LEN,
        "ALPHABET must be 85 ascii-char len"
    );
    while i < RFC1924_ALPHABET.len() {
        table[RFC1924_ALPHABET[i] as usize] = Some(i as u8);
        i += 1;
    }
    table
}

const RFC1924_DECODE: [Option<u8>; U8_COUNT_VALUE] = generate_rfc1924_decode_table();

#[inline]
fn char85_to_byte(c: u8) -> Result<u8> {
    direct_char85_to_byte(c)
}

#[inline]
fn direct_char85_to_byte(c: u8) -> Result<u8> {
    RFC1924_DECODE[c as usize].ok_or_else(|| Error::InvalidCharacter(c))
}

#[inline]
fn byte_to_char85(x85: u8) -> u8 {
    RFC1924_ALPHABET[x85 as usize]
}

/// this is no allocation api for base85 encoding and decoding. it allow to encode and decode in place,
/// so it can be used in no_std environment without heap allocation. the input and output buffers must
/// be large enough to hold the encoded or decoded data. the function will return the number of
/// bytes written to the output buffer. the input buffer can be reused after encoding or decoding.
///
/// During encoding, this function is used to calculate size to allocate for output buffer.
///
/// const function allow you to evaluate it at compile time (usefull for set buffer size for no_alloc version)
///
pub const fn calc_encode_len(indata_bytes_len: usize) -> usize {
    let chunks_num = indata_bytes_len / 4;
    let remain = indata_bytes_len - (chunks_num * 4); // Modulo is more expensive than sub, so we do it this way
    if remain == 0 {
        chunks_num * 5
    } else {
        chunks_num * 5 + remain + 1
    }
}

/// During decoding, this function is used to calculate size to allocate for output buffer.
pub const fn calc_decode_len(indata_bytes_len: usize) -> usize {
    let chunks_num = indata_bytes_len / 5;
    let remain = indata_bytes_len - (chunks_num * 5); // Mod is more expensive than sub, so we do it this way
    if remain == 0 {
        chunks_num * 4
    } else {
        chunks_num * 4 + remain - 1
    }
}

/// encode return base85 encoded data in a new allocated String
pub fn encode(indata: &[u8]) -> String {
    let final_encoded_len = calc_encode_len(indata.len());
    let mut out;

    #[cfg(not(feature = "only_safe"))]
    {
        out = Vec::with_capacity(final_encoded_len);
        unsafe {
            // No initialization of the buffer is needed, as encode_noalloc will write to the entire buffer. We can safely set the length to the capacity after encoding.
            out.set_len(final_encoded_len);
        }
    }
    #[cfg(feature = "only_safe")]
    {
        out = vec![0; final_encoded_len];
    }
    // This no unsafe variant is same speed on big size array, but slower -+70% for short (8->16bytes size ??))
    // let mut out = vec![0; encode_len];
    let _ = encode_noalloc_inner(indata, final_encoded_len, &mut out).unwrap();

    // encode_noalloc  unwrap can't failed because we pre-allocate right size
    //
    // from_utf8
    // Encoding result is always a valid UTF-8 string, so we can safely use from_utf8_unchecked here.
    // This is a micro optimization to avoid the overhead of checking for UTF-8 validity, (4% to 15% )
    // which we know is guaranteed by the encoding process.
    // unwrap can't failed because we output only utf8 char
    // from_utf_ move allocated space from out to String (no other allocation is done)
    #[cfg(not(feature = "only_safe"))]
    {
        unsafe { String::from_utf8_unchecked(out) }
    }
    #[cfg(feature = "only_safe")]
    {
        String::from_utf8(out).unwrap()
    }
}

/// encode_noalloc will encode indata to out slice.
///
/// out slice must be big enough or Error will be return.
///
/// you can use calc_encode_len() to compute need size for output buffer
///
/// returned slice reference a sub part of given out slice
pub fn encode_noalloc<'a>(indata: &[u8], out: &'a mut [u8]) -> Result<&'a str> {
    let final_encoded_len = calc_encode_len(indata.len());
    encode_noalloc_inner(indata, final_encoded_len, out)
}

/// you can use calc_encode_len() to compute need size for output buffer
fn encode_noalloc_inner<'a>(
    indata: &[u8],
    final_encoded_len: usize,
    out: &'a mut [u8],
) -> Result<&'a str> {
    let chunks = indata.chunks_exact(4);
    let remainder = chunks.remainder();
    if out.len() < final_encoded_len {
        return Err(Error::OutputBufferTooSmall);
    }
    let out = &mut out[..final_encoded_len];
    let mut out_chunks = out.chunks_exact_mut(5);

    for (chunk, out) in std::iter::zip(chunks, &mut out_chunks) {
        let decnum = u32::from_be_bytes(<[u8; 4]>::try_from(chunk).unwrap());
        out[0] = byte_to_char85((decnum / 85u32.pow(4)) as u8);
        out[1] = byte_to_char85(((decnum % 85u32.pow(4)) / 85u32.pow(3)) as u8);
        out[2] = byte_to_char85(((decnum % 85u32.pow(3)) / 85u32.pow(2)) as u8);
        out[3] = byte_to_char85(((decnum % 85u32.pow(2)) / 85u32) as u8);
        out[4] = byte_to_char85((decnum % 85u32) as u8);
    }

    let out_remainder = out_chunks.into_remainder();
    if let Some(a) = remainder.first().copied() {
        let b = remainder.get(1).copied();
        let c = remainder.get(2).copied();
        let d = remainder.get(3).copied();
        let decnum = u32::from_be_bytes([a, b.unwrap_or(0), c.unwrap_or(0), d.unwrap_or(0)]);
        out_remainder[0] = byte_to_char85((decnum / 85u32.pow(4)) as u8);
        out_remainder[1] = byte_to_char85(((decnum % 85u32.pow(4)) / 85u32.pow(3)) as u8);
        if b.is_some() {
            out_remainder[2] = byte_to_char85(((decnum % 85u32.pow(3)) / 85u32.pow(2)) as u8);
        }
        if c.is_some() {
            out_remainder[3] = byte_to_char85(((decnum % 85u32.pow(2)) / 85u32) as u8);
        }
        if d.is_some() {
            out_remainder[4] = byte_to_char85((decnum % 85u32) as u8);
        }
    }

    #[cfg(not(feature = "only_safe"))]
    unsafe {
        let r: &'a str = str::from_utf8_unchecked(&out[..final_encoded_len]);
        Ok(r)
    }
    #[cfg(feature = "only_safe")]
    {
        let r: &'a str = str::from_utf8(&out[..final_encoded_len]).unwrap();
        Ok(r)
    }
}

/// decode indata as base85 encoded to a new allocated Vec of u8
pub fn decode(indata: &[u8]) -> Result<Vec<u8>> {
    let final_decoded_len = calc_decode_len(indata.len());
    let mut out;

    #[cfg(not(feature = "only_safe"))]
    {
        out = Vec::with_capacity(final_decoded_len);
        unsafe {
            // No initialization of the buffer is needed, as decode_noalloc will write to the entire buffer. We can safely set the length to the capacity after encoding.
            out.set_len(final_decoded_len);
        }
    }
    #[cfg(feature = "only_safe")]
    {
        out = vec![0; final_decoded_len];
    }

    let _ = decode_noalloc_inner(indata, final_decoded_len, &mut out)
        .map_err(|_e| Error::UnexpectedEof)?;
    Ok(out)
}

/// decode() process indata as base85 encoded string and decode it to out slice
///
/// you can use calc_decode_len() to compute need size for output buffer
///
/// returned slice reference a sub part of given out slice
pub fn decode_noalloc<'a>(indata: &[u8], out: &'a mut [u8]) -> Result<&'a mut [u8]> {
    let final_decoded_len = calc_decode_len(indata.len());
    decode_noalloc_inner(indata, final_decoded_len, out)
}

/// decode_noalloc_inner() this is the internal function
///
/// she **assert** than final_decoded_len is result of calc_decode_len()
fn decode_noalloc_inner<'a>(
    indata: &[u8],
    final_decoded_len: usize,
    out: &'a mut [u8],
) -> Result<&'a mut [u8]> {
    let chunks = indata.chunks_exact(5);
    let remainder = chunks.remainder();
    if out.len() < final_decoded_len {
        return Err(Error::OutputBufferTooSmall);
    }
    let out = &mut out[..final_decoded_len];

    let mut out_chunks = out.chunks_exact_mut(4);

    for (chunk, out_chunk) in std::iter::zip(chunks, &mut out_chunks) {
        let accumulator = u32::from(char85_to_byte(chunk[0])?) * 85u32.pow(4)
            + u32::from(char85_to_byte(chunk[1])?) * 85u32.pow(3)
            + u32::from(char85_to_byte(chunk[2])?) * 85u32.pow(2)
            + u32::from(char85_to_byte(chunk[3])?) * 85u32
            + u32::from(char85_to_byte(chunk[4])?);
        out_chunk[0] = (accumulator >> 24) as u8;
        out_chunk[1] = (accumulator >> 16) as u8;
        out_chunk[2] = (accumulator >> 8) as u8;
        out_chunk[3] = accumulator as u8;
    }

    let out_remainder = out_chunks.into_remainder();
    if let Some(a) = remainder.first().copied() {
        let b = remainder.get(1).copied();
        let c = remainder.get(2).copied();
        let d = remainder.get(3).copied();
        let e = remainder.get(4).copied();
        let accumulator = u32::from(char85_to_byte(a)?) * 85u32.pow(4)
            + u32::from(b.map_or(Err(Error::UnexpectedEof), char85_to_byte)?) * 85u32.pow(3)
            + u32::from(c.map_or(Ok(126), char85_to_byte)?) * 85u32.pow(2)
            + u32::from(d.map_or(Ok(126), char85_to_byte)?) * 85u32.pow(1)
            + u32::from(e.map_or(Ok(126), char85_to_byte)?) * 85u32.pow(0);
        out_remainder[0] = (accumulator >> 24) as u8;
        if remainder.len() > 2 {
            out_remainder[1] = (accumulator >> 16) as u8;
            if remainder.len() > 3 {
                out_remainder[2] = (accumulator >> 8) as u8;
                if remainder.len() > 4 {
                    out_remainder[3] = accumulator as u8;
                }
            }
        }
    }

    Ok(&mut out[..final_decoded_len])
}

#[cfg(test)]
mod tests {
    #[allow(unused)]
    use super::*;
    // Check with https://nerdmosis.com/tools/encode-and-decode-base85
    const RFC1924_ALPHABET_ENCODED :&str= "FflSSG&MFiI5|N=LqtVJM@UIZOH55pPf$@(Q&d$}S6EqEVPa!sWoBn+X=-b1ZEkOHadLBXb#`}nd3qruBqb&&DJm;1J3Ku;KR{kzV0(Oheg";
    const fn get_rfc1924_dic_as_str() -> &'static str {
        // We build out table with this guarranted (only utf8 char), then it's safe for use to use unsafe here
        unsafe { std::str::from_utf8_unchecked(crate::RFC1924_ALPHABET) }
    }
    const TESTLIST: [(&str, &str); 9] = [
        ("a", "VE"),
        ("aa", "VPO"),
        ("aaa", "VPRn"),
        ("aaaa", "VPRom"),
        ("aaaaa", "VPRomVE"),
        ("aaaaaa", "VPRomVPO"),
        ("aaaaaaa", "VPRomVPRn"),
        ("aaaaaaaa", "VPRomVPRom"),
        (get_rfc1924_dic_as_str(), RFC1924_ALPHABET_ENCODED),
    ];

    mod api_tests {
        #[allow(unused)]
        use super::*;
        use anyhow::Result;

        #[test]
        fn api_encode_decode() -> Result<()> {
            // The list of tests consists of the unencoded data on the left and the encoded data on
            // the right. By using strings for the arbitrary binary data, we make the test much less
            // complicated to write.

            for test in TESTLIST.iter() {
                let s = encode(test.0.as_bytes());
                assert_eq!(
                    s, test.1,
                    "encoder test failed: wanted: {}, got: {}",
                    test.0, s
                );

                let b = decode(test.1.as_bytes())
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
            Ok(())
        }

        #[test]
        fn api_encode_noalloc_and_decode_noalloc() -> Result<()> {
            // The list of tests consists of the decoded data on the left and the encoded data on
            // the right. By using strings for the arbitrary binary data, we make the test much less
            // complicated to write.

            let max_len = TESTLIST.iter().fold(0 as usize, |acc, test| {
                calc_encode_len(test.0.len()).max(acc)
            });
            assert_eq!(107, max_len, "test data is too long for the output buffer");
            let mut output_orig = vec![0u8; max_len];
            let mut output = &mut output_orig[..];
            for test in TESTLIST.iter() {
                let resu = encode_noalloc(test.0.as_bytes(), &mut output)?;

                assert_eq!(
                    test.1.as_bytes(),
                    resu.as_bytes(),
                    "encoder test failed: wanted: {:?}, got: {:?}",
                    test.0.as_bytes(),
                    resu
                );

                let b = decode_noalloc(test.1.as_bytes(), &mut output)
                    .unwrap_or_else(|e| panic!("decoder test error on input {}: {}", test.1, e));

                let s = str::from_utf8(b).unwrap_or_else(|e| {
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
            Ok(())
        }
    }

    mod unit_tests {
        #[allow(unused)]
        use super::*;
        use anyhow::Result;

        #[test]
        fn unit_check_alphabet_used() -> Result<()> {
            assert_eq!(85, crate::RFC1924_ALPHABET.len());
            Ok(())
        }

        #[test]
        fn unit_count_char_in_decode_table() -> Result<()> {
            let count_char_in_decode_table =
                crate::RFC1924_DECODE.iter().fold(0_u32, |count, x| {
                    if x.is_some() {
                        count + 1
                    } else {
                        count
                    }
                });
            assert_eq!(85, count_char_in_decode_table);
            Ok(())
        }

        #[test]
        fn unit_check_len() -> Result<()> {
            for (input, expected_output) in TESTLIST.iter() {
                assert_eq!(input.len(), calc_decode_len(expected_output.len()));
                assert_eq!(expected_output.len(), calc_encode_len(input.len()));
            }
            Ok(())
        }

        #[test]
        fn unit_encode_and_decode_all_possible_chars() -> Result<()> {
            let mut input = Vec::<u8>::with_capacity(256);
            for i in 0..=255 {
                input.push(i as u8);
            }
            let encoded = encode(&input);
            let all_possible_encoded:&str="009C61O)~M2nh-c3=Iws5D^j+6crX17#SKH9337XAR!_nBqb&%C@Cr{EG;fCFflSSG&MFiI5|2yJUu=?KtV!7L`6nNNJ&adOifNtP*GA-R8>}2SXo+ITwPvYU}0ioWMyV&XlZI|Y;A6DaB*^Tbai%jczJqze0_d@fPsR8goTEOh>41ejE#<ukdcy;l$Dm3n3<ZJoSmMZprN9pq@|{(sHv)}tgWuEu(7hUw6(UkxVgH!yuH4^z`?@9#Kp$P$jQpf%+1cv(9zP<)YaD4*xB0K+}+;a;Njxq<mKk)=;`X~?CtLF@bU8V^!4`l`1$(#{Qds_";
            assert_eq!(all_possible_encoded, encoded);

            let decoded = decode(encoded.as_bytes())?;
            assert_eq!(input, decoded);
            Ok(())
        }
    }
}
