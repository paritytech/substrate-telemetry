use std::fmt::{self, Debug};

use serde::de::{self, Deserialize, Deserializer, Unexpected, Visitor};

const HASH_BYTES: usize = 32;

/// Newtype wrapper for 32-byte hash values, implementing readable `Debug` and `serde::Deserialize`.
// We could use primitive_types::H256 here, but opted for a custom type to aboid more dependencies.
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct Hash([u8; HASH_BYTES]);

struct HashVisitor;

impl<'de> Visitor<'de> for HashVisitor {
    type Value = Hash;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("hexidecimal string of 32 bytes beginning with 0x")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if !value.starts_with("0x") {
            return Err(de::Error::invalid_value(Unexpected::Str(value), &self));
        }

        let mut hash = [0; HASH_BYTES];

        hex::decode_to_slice(&value[2..], &mut hash)
            .map_err(|_| de::Error::invalid_value(Unexpected::Str(value), &self))?;

        Ok(Hash(hash))
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Hash, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(HashVisitor)
    }
}

impl Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("0x")?;

        let mut ascii = [0; HASH_BYTES * 2];

        hex::encode_to_slice(self.0, &mut ascii)
            .expect("Encoding 32 bytes into 64 bytes of ascii; qed");

        f.write_str(std::str::from_utf8(&ascii).expect("ASCII hex encoded bytes canot fail; qed"))
    }
}
