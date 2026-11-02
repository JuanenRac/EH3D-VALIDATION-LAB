// SPDX-License-Identifier: GPL-3.0-or-later
//! SHA-256 of bytes and of files, as 64 lower-case hexadecimal digits.

use sha2::{Digest, Sha256};
use std::io::Read;

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(DIGITS[usize::from(byte >> 4)]));
        out.push(char::from(DIGITS[usize::from(byte & 15)]));
    }
    out
}

/// SHA-256 of a byte slice.
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

/// SHA-256 of everything a reader yields, read in blocks so a large file never sits in memory.
pub fn sha256_reader<R: Read>(mut reader: R) -> std::io::Result<String> {
    let mut hasher = Sha256::new();
    let mut block = vec![0u8; 64 * 1024];
    loop {
        let read = reader.read(&mut block)?;
        if read == 0 {
            break;
        }
        hasher.update(&block[..read]);
    }
    Ok(hex(&hasher.finalize()))
}

/// True when the text is exactly 64 lower-case hexadecimal digits.
pub fn is_sha256_hex(text: &str) -> bool {
    text.len() == 64
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// The opaque 16-digit stand-in for a name in a report: the start of the SHA-256 of the salt, a zero byte and the name.
pub fn opaque_id(salt: &str, name: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update([0u8]);
    hasher.update(name.as_bytes());
    let digest = hex(&hasher.finalize());
    digest[..16].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Published test vectors of FIPS 180-4 / NIST CAVP.
    #[test]
    fn matches_the_published_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn the_streaming_hash_agrees_with_the_slice_hash_across_block_borders() {
        let data: Vec<u8> = (0..200_000u32).map(|n| (n % 251) as u8).collect();
        assert_eq!(sha256_reader(&data[..]).unwrap(), sha256_hex(&data));
    }

    #[test]
    fn hex_checker_is_strict() {
        assert!(is_sha256_hex(&sha256_hex(b"x")));
        assert!(!is_sha256_hex(&sha256_hex(b"x").to_uppercase()));
        assert!(!is_sha256_hex("abc"));
    }

    #[test]
    fn the_opaque_id_depends_on_the_salt_and_hides_the_name() {
        let plain = opaque_id("", "board-7");
        assert_eq!(plain.len(), 16);
        assert_ne!(plain, opaque_id("pepper", "board-7"));
        assert!(!plain.contains("board"));
    }
}
