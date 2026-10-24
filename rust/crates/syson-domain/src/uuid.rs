// Copyright (c) 2024, 2026 Obeo.
// This program and accompanying materials are made available under the terms
// of the Eclipse Public License v2.0, https://www.eclipse.org/legal/epl-2.0/.
// SPDX-License-Identifier: EPL-2.0
// Contributors: Obeo - initial API and implementation
// Rust translation and compatibility tests: SysON Rust port contributors.

//! ElementUtil UUIDv5 deterministic identity primitive (not an authentication hash).
use std::fmt;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid(pub [u8; 16]);
impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, b) in self.0.iter().enumerate() {
            if [4, 6, 8, 10].contains(&i) {
                f.write_str("-")?;
            }
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}
pub const NAMESPACE_URL: Uuid = Uuid([
    0x6b, 0xa7, 0xb8, 0x11, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
]);
pub const KERML_LIBRARY_BASE_URI: &str = "https://www.omg.org/spec/KerML/";
pub const SYSML_LIBRARY_BASE_URI: &str = "https://www.omg.org/spec/SysML/";
/// Deterministic identity from namespace bytes and UTF-8 name, as upstream.
pub fn uuid_v5(namespace: Uuid, value: &str) -> Uuid {
    let mut bytes = namespace.0.to_vec();
    bytes.extend_from_slice(value.as_bytes());
    let hash = sha1(&bytes);
    let mut result = [0u8; 16];
    result.copy_from_slice(&hash[..16]);
    result[6] = (result[6] & 0x0f) | 0x50;
    result[8] = (result[8] & 0x3f) | 0x80;
    Uuid(result)
}
/// Upstream checks string prefixes rather than strictly parsed URI schemes.
pub fn is_standard_library_resource(uri: Option<&str>) -> bool {
    uri.is_some_and(|u| u.starts_with("kermllibrary") || u.starts_with("sysmllibrary"))
}
/// Deterministic branch for standard LibraryPackage; other UUID decisions remain
/// with the resource/model adapter (this function never generates random IDs).
pub fn standard_library_uuid(resource_uri: &str, qualified_name: &str) -> Uuid {
    let base = if resource_uri.starts_with("kermllibrary:") {
        KERML_LIBRARY_BASE_URI
    } else {
        SYSML_LIBRARY_BASE_URI
    };
    uuid_v5(NAMESPACE_URL, &format!("{base}{qualified_name}"))
}
fn sha1(input: &[u8]) -> [u8; 20] {
    let mut data = input.to_vec();
    let bit_len = (data.len() as u64).wrapping_mul(8);
    data.push(0x80);
    while data.len() % 64 != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());
    let mut h: [u32; 5] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0];
    for chunk in data.as_chunks::<64>().0 {
        let mut w = [0u32; 80];
        for (i, bytes) in chunk.as_chunks::<4>().0.iter().enumerate() {
            w[i] = u32::from_be_bytes(*bytes);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let [mut a, mut b, mut c, mut d, mut e] = h;
        for (i, word) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | (!b & d), 0x5a827999u32),
                20..=39 => (b ^ c ^ d, 0x6ed9eba1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1bbcdc),
                _ => (b ^ c ^ d, 0xca62c1d6),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        for (dst, value) in h.iter_mut().zip([a, b, c, d, e]) {
            *dst = dst.wrapping_add(value);
        }
    }
    let mut out = [0; 20];
    for (bytes, value) in out.as_chunks_mut::<4>().0.iter_mut().zip(h) {
        bytes.copy_from_slice(&value.to_be_bytes());
    }
    out
}
