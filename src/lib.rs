use base64;
use uuid;

/// Return a randomly-generated slugid.
pub fn v4() -> String {
    let mut enc = base64::encode_config(uuid::Uuid::new_v4().as_bytes(), base64::URL_SAFE);
    enc.truncate(22); // strip trailing ==
    enc
}

/// In the rust implementation, (TODO: currently the same as v4())
pub fn nice() -> String {
    let mut bytes = *uuid::Uuid::new_v4().as_bytes();
    bytes[0] = bytes[0] & 0x7f; // unset first bit to ensure [A-Za-f] first char
    let mut enc = base64::encode_config(bytes, base64::URL_SAFE);
    enc.truncate(22); // strip trailing ==
    enc
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
    }
}
