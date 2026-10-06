//! Artifact identifiers and content identity (decision D007).
//!
//! A sequence artifact has two identities, and they must not be conflated:
//!
//! * an `Identifier`, chosen by the caller, which distinguishes one artifact
//!   record from another;
//! * a `ContentHash`, computed from the canonical sequence bytes only.
//!
//! A `SequenceReference` pairs the two. Structures that point at a sequence
//! (intervals, features, CDSs) carry a `SequenceReference`, never a bare
//! display name, so a same-name/different-content artifact cannot be
//! substituted silently.
//!
//! The hash contract is deliberately narrow:
//!
//! ```text
//! SHA-256 over the ASCII bytes of the normalized (canonical) sequence text,
//! with no separators, no header, no display name, no topology, and no other
//! metadata.
//! ```
//!
//! Two artifacts with the same canonical bases therefore share a content
//! hash even when their identifiers or display names differ, and two
//! artifacts with different canonical bases never share one. Source
//! metadata is never part of content identity.

use crate::error::DomainError;
use crate::error::InvalidIdentifierProblem;

/// A stable artifact identifier.
#[derive(Clone, Debug, PartialEq)]
pub struct Identifier {
    value: String,
}

/// A SHA-256 digest rendered as lowercase hexadecimal.
#[derive(Clone, Debug, PartialEq)]
pub struct ContentHash {
    hex_digest: String,
}

impl ContentHash {
    /// Wrap a lowercase hexadecimal digest.
    pub fn create(hex_digest: String) -> ContentHash {
        ContentHash { hex_digest }
    }

    pub fn hex_digest(&self) -> &str {
        self.hex_digest.as_str()
    }
}

/// A reference to a sequence artifact: display name plus content digest.
///
/// Equality is identity equality: two references are equal only when both
/// the display identifier and the content digest agree. Structures that
/// point at a sequence carry this value so that a different artifact that
/// happens to share a display name is never accepted silently.
#[derive(Clone, Debug, PartialEq)]
pub struct SequenceReference {
    name: Identifier,
    content_hash: ContentHash,
}

impl SequenceReference {
    pub fn create(name: Identifier, content_hash: ContentHash) -> SequenceReference {
        SequenceReference { name, content_hash }
    }

    pub fn name(&self) -> Identifier {
        self.name.clone()
    }

    pub fn content_hash(&self) -> ContentHash {
        self.content_hash.clone()
    }

    /// Short human-readable form used in error messages: `name@digest8`.
    pub fn describe(&self) -> String {
        let chars: Vec<char> = self
            .content_hash()
            .hex_digest()
            .to_string()
            .chars()
            .collect();
        let mut prefix: String = String::new();
        let limit: usize = chars.len();
        let mut index: usize = 0;
        while index < limit && index < 8 {
            prefix.push(chars[index]);
            index += 1;
        }
        self.name().value().to_string() + "@" + &prefix
    }
}

impl Identifier {
    /// Validate and construct an identifier.
    ///
    /// An identifier must be a single printable ASCII token: non-empty, with
    /// no whitespace and no control characters.
    pub fn create(value: String) -> Result<Identifier, DomainError> {
        if value.is_empty() {
            return Err(DomainError::EmptyIdentifier);
        }
        for character in value.chars() {
            if character <= ' ' || character > '~' {
                return Err(DomainError::InvalidIdentifierCharacters(
                    InvalidIdentifierProblem {
                        identifier: value,
                        reason: "must be a single printable ASCII token without whitespace"
                            .to_string(),
                    },
                ));
            }
        }
        Ok(Identifier { value })
    }

    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

/// The 64 SHA-256 round constants.
const SHA256_ROUND_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// The initial SHA-256 state.
const SHA256_INITIAL_STATE: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

fn rotate_right(value: u32, bits: u32) -> u32 {
    value.rotate_right(bits)
}

fn choice(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ ((!x) & z)
}

fn majority(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

fn small_sigma(value: u32) -> u32 {
    rotate_right(value, 7) ^ rotate_right(value, 18) ^ (value >> 3)
}

fn small_sigma_two(value: u32) -> u32 {
    rotate_right(value, 17) ^ rotate_right(value, 19) ^ (value >> 10)
}

fn big_sigma(value: u32) -> u32 {
    rotate_right(value, 2) ^ rotate_right(value, 13) ^ rotate_right(value, 22)
}

fn big_sigma_two(value: u32) -> u32 {
    rotate_right(value, 6) ^ rotate_right(value, 11) ^ rotate_right(value, 25)
}

/// SHA-256 of `data`, lowercase hexadecimal.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut state = SHA256_INITIAL_STATE;
    let bit_length: u64 = (data.len() as u64) * 8;
    let mut message = data.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    for byte in bit_length.to_be_bytes() {
        message.push(byte);
    }

    let mut offset: usize = 0;
    while offset < message.len() {
        let block = &message[offset..offset + 64];
        offset += 64;

        let mut words = [0u32; 64];
        for index in 0..16 {
            words[index] = ((block[index * 4] as u32) << 24)
                | ((block[index * 4 + 1] as u32) << 16)
                | ((block[index * 4 + 2] as u32) << 8)
                | (block[index * 4 + 3] as u32);
        }
        for index in 16..64 {
            words[index] = small_sigma_two(words[index - 2])
                .wrapping_add(words[index - 7])
                .wrapping_add(small_sigma(words[index - 15]))
                .wrapping_add(words[index - 16]);
        }

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];

        for index in 0..64 {
            let temp_one = h
                .wrapping_add(big_sigma_two(e))
                .wrapping_add(choice(e, f, g))
                .wrapping_add(SHA256_ROUND_CONSTANTS[index])
                .wrapping_add(words[index]);
            let temp_two = big_sigma(a).wrapping_add(majority(a, b, c));
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp_one);
            d = c;
            c = b;
            b = a;
            a = temp_one.wrapping_add(temp_two);
        }

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }

    let mut hex = String::new();
    for word in state {
        hex += format!("{:08x}", word).as_str();
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCategory;

    // Digests below were produced with Python's hashlib (an independent
    // SHA-256 implementation) and are recorded in the fixture documentation.
    #[test]
    fn empty_input_matches_the_published_vector() {
        assert!(
            sha256_hex("".as_bytes())
                == "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn short_input_matches_the_published_vector() {
        assert!(
            sha256_hex("abc".as_bytes())
                == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn multi_block_input_matches_hashlib() {
        let filler = "x".repeat(1000);
        assert!(
            sha256_hex(filler.as_bytes())
                == "44f8354494a5ba03ba1792a8d3e9c534c47a9181980fde7a3f44b06ef2ae7c7f"
        );
    }

    #[test]
    fn pangram_matches_hashlib() {
        assert!(
            sha256_hex("The quick brown fox jumps over the lazy dog".as_bytes())
                == "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592"
        );
    }

    #[test]
    fn content_identity_ignores_the_display_name() {
        let first = Identifier::create("seq-a".to_string());
        let second = Identifier::create("seq-a-copy".to_string());
        assert!(first.is_ok());
        assert!(second.is_ok());
        assert!(
            sha256_hex("ATGGCTTGGAATCAGCAGCAGCAGCAGCAG".as_bytes())
                == "a83f6d825c144cef563438c085821ecbd8986cdaf0f65c75f7994ed616a2fd80"
        );
    }

    #[test]
    fn identifier_rules_are_enforced() {
        assert!(Identifier::create("seq-a".to_string()).is_ok());
        assert!(Identifier::create("".to_string()).unwrap_err() == DomainError::EmptyIdentifier);
        let with_space = Identifier::create("seq a".to_string());
        assert!(with_space.unwrap_err().category() == ErrorCategory::MalformedInput);
    }

    // Padding boundary: a message whose final block is nearly full of length
    // bytes still has room for the 0x80 byte and the 8-byte bit length.
    #[test]
    fn padding_boundary_blocks_match_hashlib() {
        let pattern = "abcdefghijklmnopqrstuvwxyz";
        let repeated = pattern.repeat(10);
        assert!(
            sha256_hex(&repeated.as_bytes()[0..55])
                == "595615dbe4f0f407ae397d08b4c2cb870cb9b0e11937416f950c5160acf9c005"
        );
        assert!(
            sha256_hex(&repeated.as_bytes()[0..56])
                == "784f623b787495078e93ff28a25b581df0584055a7e71d8cd90c454716b92f51"
        );
        assert!(
            sha256_hex(&repeated.as_bytes()[0..63])
                == "5ca3e1ef5207490eac01a795e5cc94d59582a5118bf9534665c8668d87aa647c"
        );
        assert!(
            sha256_hex(&repeated.as_bytes()[0..64])
                == "2fcd5a0d60e4c941381fcc4e00a4bf8be422c3ddfafb93c809e8d1e2bfffae8e"
        );
        assert!(
            sha256_hex(&repeated.as_bytes()[0..65])
                == "1b3cd1877ab2f2f19f7be001722554f336cb799df0329de0bb4c118dc6abc06d"
        );
    }

    #[test]
    fn sequence_reference_equality_requires_name_and_content() {
        let digest_a: ContentHash =
            ContentHash::create(sha256_hex("ATGGCTTGGAATCAGCAGCAGCAGCAGCAG".as_bytes()));
        let digest_b: ContentHash =
            ContentHash::create(sha256_hex("ATTCCAAGCCATCAGCAGCAGCAGCAGCAG".as_bytes()));
        let built = Identifier::create("seq-a".to_string());
        assert!(built.is_ok());
        let name: Identifier = built.unwrap();
        let other_built = Identifier::create("seq-b".to_string());
        assert!(other_built.is_ok());
        let other: Identifier = other_built.unwrap();
        let first = SequenceReference::create(name.clone(), digest_a.clone());
        let same_name_other_content = SequenceReference::create(name.clone(), digest_b.clone());
        let other_name = SequenceReference::create(other.clone(), digest_a.clone());
        assert!(first == first);
        assert!(first != same_name_other_content);
        assert!(first != other_name);
    }

    #[test]
    fn sequence_reference_description_is_short_and_stable() {
        let built = Identifier::create("seq-a".to_string());
        assert!(built.is_ok());
        let name: Identifier = built.unwrap();
        let reference = SequenceReference::create(
            name.clone(),
            ContentHash::create(sha256_hex("ATGGCTTGGAATCAGCAGCAGCAGCAGCAG".as_bytes())),
        );
        assert!(reference.describe() == "seq-a@a83f6d82");
    }
}
