//! A hash wrapper which can be deserialized from a hex string as well as from an array of bytes,
//! so that it can deal with the sort of inputs we expect from substrate nodes.

use serde::de::{self, Deserialize, Deserializer, SeqAccess, Unexpected, Visitor};
use serde::ser::{Serialize, Serializer};
use std::fmt::{self, Debug, Display};
use std::str::FromStr;

/// We assume that hashes are 32 bytes long, and in practise that's currently true,
/// but in theory it doesn't need to be. We may need to be more dynamic here.
const HASH_BYTES: usize = 32;

/// Newtype wrapper for 32-byte hash values, implementing readable `Debug` and `serde::Deserialize`.
/// This can deserialize from a JSON string or array.
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct Hash([u8; HASH_BYTES]);

impl From<Hash> for common::node_types::BlockHash {
    fn from(hash: Hash) -> Self {
        hash.0.into()
    }
}

impl From<common::node_types::BlockHash> for Hash {
    fn from(hash: common::node_types::BlockHash) -> Self {
        Hash(hash.0)
    }
}

struct HashVisitor;

impl<'de> Visitor<'de> for HashVisitor {
    type Value = Hash;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(
            "byte array of length 32, or hexidecimal string of 32 bytes beginning with 0x",
        )
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        value
            .parse()
            .map_err(|_| de::Error::invalid_value(Unexpected::Str(value), &self))
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value.len() == HASH_BYTES {
            let mut hash = [0; HASH_BYTES];

            hash.copy_from_slice(value);

            return Ok(Hash(hash));
        }

        Hash::from_ascii(value)
            .map_err(|_| de::Error::invalid_value(Unexpected::Bytes(value), &self))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut hash = [0u8; HASH_BYTES];

        for (i, byte) in hash.iter_mut().enumerate() {
            match seq.next_element()? {
                Some(b) => *byte = b,
                None => return Err(de::Error::invalid_length(i, &"an array of 32 bytes")),
            }
        }

        if seq.next_element::<u8>()?.is_some() {
            return Err(de::Error::invalid_length(33, &"an array of 32 bytes"));
        }

        Ok(Hash(hash))
    }
}

impl Hash {
    pub fn from_ascii(value: &[u8]) -> Result<Self, HashParseError> {
        if !value.starts_with(b"0x") {
            return Err(HashParseError::InvalidPrefix);
        }

        let mut hash = [0; HASH_BYTES];

        hex::decode_to_slice(&value[2..], &mut hash).map_err(HashParseError::HexError)?;

        Ok(Hash(hash))
    }
}

impl FromStr for Hash {
    type Err = HashParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Hash::from_ascii(value.as_bytes())
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Hash, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(HashVisitor)
    }
}

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("0x")?;

        let mut ascii = [0; HASH_BYTES * 2];

        hex::encode_to_slice(self.0, &mut ascii)
            .expect("Encoding 32 bytes into 64 bytes of ascii; qed");

        f.write_str(std::str::from_utf8(&ascii).expect("ASCII hex encoded bytes can't fail; qed"))
    }
}

impl Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum HashParseError {
    #[error("Error parsing string into hex: {0}")]
    HexError(hex::FromHexError),
    #[error("Invalid hex prefix: expected '0x'")]
    InvalidPrefix,
}

#[cfg(test)]
mod tests {
    use super::Hash;
    use bincode::Options;

    const DUMMY: Hash = {
        let mut hash = [0; 32];
        hash[0] = 0xDE;
        hash[1] = 0xAD;
        hash[2] = 0xBE;
        hash[3] = 0xEF;
        Hash(hash)
    };

    #[test]
    fn deserialize_json_hash_str() {
        let json = r#""0xdeadBEEF00000000000000000000000000000000000000000000000000000000""#;

        let hash: Hash = serde_json::from_str(json).unwrap();

        assert_eq!(hash, DUMMY);
    }

    #[test]
    fn deserialize_json_array() {
        let json = r#"[222,173,190,239,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]"#;

        let hash: Hash = serde_json::from_str(json).unwrap();

        assert_eq!(hash, DUMMY);
    }

    #[test]
    fn deserialize_json_array_too_short() {
        let json = r#"[222,173,190,239,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]"#;

        let res = serde_json::from_str::<Hash>(json);

        assert!(res.is_err());
    }

    #[test]
    fn deserialize_json_array_too_long() {
        let json = r#"[222,173,190,239,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]"#;

        let res = serde_json::from_str::<Hash>(json);

        assert!(res.is_err());
    }

    #[test]
    fn bincode() {
        let bytes = bincode::options().serialize(&DUMMY).unwrap();

        let mut expected = [0; 33];

        expected[0] = 32; // length
        expected[1..].copy_from_slice(&DUMMY.0);

        assert_eq!(bytes, &expected);

        let deserialized: Hash = bincode::options().deserialize(&bytes).unwrap();

        assert_eq!(DUMMY, deserialized);
    }
}
