//! Generate slugds.
//!
//! Slugids are fixed-length (22 characters) URL-safe random identifiers.  They contain enough
//! entropy that for all practical purposes they can be considered unique.
//!
//! See https://github.com/taskcluster/slugid for details.
use base64;
use lazy_static::lazy_static;
use ring::rand::{SecureRandom, SystemRandom};

lazy_static! {
    static ref RNG: SystemRandom = SystemRandom::new();
}

/// Generate a 16-byte representation of a v4 UUID.  We do not use the uuid crate because it does
/// not use a CSPRNG.
#[inline(always)]
fn uuid_v4(rng: &dyn SecureRandom) -> [u8; 16] {
    let mut bytes = [0u8; 16];
    rng.fill(&mut bytes)
        .expect("could not generate random values");
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // RFC4122
    bytes[6] = (bytes[6] & 0x0f) | 0x40; // Random
    bytes
}

/// Encode a UUID as a slugid (base64, url-safe, without padding)
#[inline(always)]
fn encode(bytes: &[u8; 16]) -> String {
    base64::encode_config(bytes, base64::URL_SAFE_NO_PAD)
}

/// Like `v4` but accepting a ring rng as a source of randomness.
pub fn v4_rng(rng: &dyn SecureRandom) -> String {
    let bytes = uuid_v4(rng);
    encode(&bytes)
}

/// Like `nice` but accepting a ring rng as a source of randomness.
pub fn nice_rng(rng: &dyn SecureRandom) -> String {
    let mut bytes = uuid_v4(rng);
    bytes[0] = bytes[0] & 0x7f; // unset first bit to ensure [A-Za-f] first char (niceness)
    encode(&bytes)
}

/// Return a randomly-generated slugid.
pub fn v4() -> String {
    v4_rng(&*RNG)
}

/// Return a randomly-generated slugid that does not begin with `-`.  This is "nicer" in the sense
/// that it is easily uesed on the command line.
pub fn nice() -> String {
    nice_rng(&*RNG)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    /* These "spread" tests check the distribution of characters in slugids; see
     * https://github.com/taskcluster/slugid/blob/master/slugid_test.js
     * for the background. */

    const CHARS_ALL: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    const CHARS_C: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdef";
    const CHARS_D: &str = "QRST";
    const CHARS_E: &str = "CGKOSWaeimquy26-";
    const CHARS_F: &str = "AQgw";

    fn spread_test(generator: fn() -> String, expected: [&str; 22]) {
        let expected: Vec<HashSet<char>> = expected
            .iter()
            .map(|s| {
                let mut set: HashSet<char> = HashSet::new();
                for c in s.chars() {
                    set.insert(c);
                }
                return set;
            })
            .collect();
        let mut got: Vec<HashSet<char>> = vec![HashSet::new(); 22];

        // call the generator 64 * 40 times, tracking the characters seen at each position.
        for _ in 0..(64 * 40) {
            let slugid = generator();
            for (i, c) in slugid.char_indices() {
                got[i].insert(c);
            }
        }

        assert_eq!(got, expected);
    }

    #[test]
    fn v4_spread_test() {
        // expected character categories for each position in the 22-character slugid
        let expected = [
            CHARS_ALL, CHARS_ALL, CHARS_ALL, CHARS_ALL, CHARS_ALL, CHARS_ALL, CHARS_ALL, CHARS_ALL,
            CHARS_D, CHARS_ALL, CHARS_E, CHARS_ALL, CHARS_ALL, CHARS_ALL, CHARS_ALL, CHARS_ALL,
            CHARS_ALL, CHARS_ALL, CHARS_ALL, CHARS_ALL, CHARS_ALL, CHARS_F,
        ];

        spread_test(v4, expected);
        spread_test(|| v4_rng(&SystemRandom::new()), expected);
    }

    #[test]
    fn nice_spread_test() {
        // expected character categories for each position in the 22-character *nice* slugid
        let expected = [
            CHARS_C, CHARS_ALL, CHARS_ALL, CHARS_ALL, CHARS_ALL, CHARS_ALL, CHARS_ALL, CHARS_ALL,
            CHARS_D, CHARS_ALL, CHARS_E, CHARS_ALL, CHARS_ALL, CHARS_ALL, CHARS_ALL, CHARS_ALL,
            CHARS_ALL, CHARS_ALL, CHARS_ALL, CHARS_ALL, CHARS_ALL, CHARS_F,
        ];

        spread_test(nice, expected);
        spread_test(|| nice_rng(&SystemRandom::new()), expected);
    }
}
