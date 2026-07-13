# a fast base85 / RFC 1924 encoding/decoding

A library for Base85 encoding as described in RFC1924 and released under the Mozilla Public License 2.0. The official repository for this crate is at [GitLab](https://gitlab.com/darkwyrm/base85). Please submit all issues and PRs to that location.

## Description

Several variants of Base85 encoding exist. The most popular variant is often known also as ascii85 and is best known for use in Adobe products. This is not that algorithm.

The variant implemented in RFC 1924 was originally intended for encoding IPv6 addresses. It utilizes the same concepts as other versions, but uses a character set which is friendly toward embedding in source code without the need for escaping. During decoding ASCII whitespace (\n, \r, \t, space) is ignored. A base85-encoded string is 25% larger than the original binary data, which is more efficient than the more-common base64 algorithm (33%). This encoding pairs very well with JSON, yielding lower overhead and needing no character escapes.

I use a pre-computed table for speedup (x26) on decoding part.

## Usage

There is 2 levels api :

### a simple API with **memory allocation**

* `encode()` turns a slice of bytes into a String and
* `decode()` turns a string reference into a Vector of bytes (u8). Both calls work completely within RAM, so processing huge files is probably not a good idea.

### a **non memory allocation api**, who use a pre-allocated buffer for result

* `calc_encode_len()` compute encoded len from a source
* `encode_noalloc()` encode to a preallocated buffer
* `calc_decode_len()` for compute decoded len from a encoded source
* `decode_noalloc()` decode to a preallocated buffer

## safe / unsafe code

* Because during encode/decode, we know final size, we can allocate right size and set size to final value directly. This is unsafe code
* If you absolutely, preferred **only_safe** code, activate feature only_safe. This limit performance by 3 to 8%, (more on very small array)
* Try bench with/without : ```cargo bench --features only_safe``` or ```cargo bench``` to compare in your usuals cases

## Contributions

I've been coding for a while, but I'm still a beginner at Rust. Suggestions and contributions are always welcome.
