# base85

A library for Base85 encoding as described in RFC1924 and released under the Mozilla Public License 2.0.

## Description

Several variants of Base85 encoding exist. The most popular variant is often known also as ascii85 and is best known for use in Adobe products. This is not that algorithm.

The variant implemented in RFC 1924 was originally intended for encoding IPv6 addresses. It utilizes the same concepts as other versions, but uses a character set which is friendly toward embedding in source code without the need for escaping. During decoding ASCII whitespace (\n, \r, \t, space) is ignored. A base85-encoded string is 25% larger than the original binary data, which is more efficient than the more-common base64 algorithm (33%). This encoding pairs very well with JSON, yielding lower overhead and needing no character escapes.

## Usage

Although this code is my first real Rust code, the code is well-tested and the API is simple: `encode()` turns a slice of bytes into a String and `decode()` turns a string reference into a Vector of bytes (u8). Both calls work completely within RAM, so processing huge files is probably not a good idea.

## Contributions

I've been coding for a while, but I'm still a beginner at Rust. Suggestions and contributions are always welcome.
