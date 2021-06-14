use serde::de::{Deserialize, Deserializer, IgnoredAny};
use serde::ser::{Serialize, Serializer};

/// Alternative to `serde::de::IgnoreAny` that implements `Serialize`.
/// Will serialize to `null` in JSON, or empty data in bincode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NullAny;

impl<'de> Deserialize<'de> for NullAny {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<NullAny, D::Error>
    where
        D: Deserializer<'de>,
    {
        // `bincode` is going to throw an error here as it does not support `IgnoredAny`.
        //
        // When using `bincode` `NullAny` will always serialize to unit (aka no data), so
        // this safely becomes a no-op.
        let _ = deserializer.deserialize_ignored_any(IgnoredAny);

        Ok(NullAny)
    }
}

impl Serialize for NullAny {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_unit()
    }
}

#[cfg(test)]
mod tests {
    use super::NullAny;
    use bincode::Options;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Dummy {
        ignore: NullAny,
    }

    #[test]
    fn deserialize_json_null() {
        let dummy: Dummy = serde_json::from_str(r#"{"ignore":null}"#).unwrap();

        assert_eq!(dummy, Dummy { ignore: NullAny });
    }

    #[test]
    fn deserialize_json_struct() {
        let dummy: Dummy = serde_json::from_str(r#"{"ignore":{"foo":"bar"}}"#).unwrap();

        assert_eq!(dummy, Dummy { ignore: NullAny });
    }

    #[test]
    fn deserialize_json_struct_invalid() {
        let dummy = serde_json::from_str::<Dummy>(r#"{"ignore":{"foo":"bar"}"#);

        assert!(dummy.is_err());
    }

    #[test]
    fn deserialize_json_vec_any() {
        let raw = [NullAny; 10];
        let json = r#"[null,true,false,10,{},[],[null],{"foo":"bar"},[9,9,9],"ten"]"#;

        let deserialized: Vec<NullAny> = serde_json::from_str(json).unwrap();

        assert_eq!(&raw[..], &deserialized);
    }

    #[test]
    fn serialize_json_null() {
        let dummy = Dummy { ignore: NullAny };

        let json = serde_json::to_string(&dummy).unwrap();

        assert_eq!(json, r#"{"ignore":null}"#);
    }

    #[test]
    fn bincode_vec() {
        let raw = vec![NullAny; 10];

        let bytes = bincode::options().serialize(&raw).unwrap();

        assert_eq!(bytes, &[10u8]);

        let deserialized: Vec<NullAny> = bincode::options().deserialize(&bytes).unwrap();

        assert_eq!(raw, deserialized);
    }

    #[test]
    fn bincode_tuple() {
        let raw = (NullAny, "Hello world".to_string());

        let bytes = bincode::options().serialize(&raw).unwrap();

        assert_eq!(bytes, b"\x0BHello world"); // 0B = 11 = length of string

        let deserialized: (NullAny, String) = bincode::options().deserialize(&bytes).unwrap();

        assert_eq!(raw, deserialized);
    }

    #[test]
    fn json_vec() {
        let raw = vec![NullAny; 10];

        let json = serde_json::to_string(&raw).unwrap();

        assert_eq!(json, "[null,null,null,null,null,null,null,null,null,null]");

        let deserialized: Vec<NullAny> = serde_json::from_str(&json).unwrap();

        assert_eq!(raw, deserialized);
    }

    #[test]
    fn json_tuple() {
        let raw = (NullAny, "Hello world".to_string());

        let json = serde_json::to_string(&raw).unwrap();

        assert_eq!(json, r#"[null,"Hello world"]"#);

        let deserialized: (NullAny, String) = serde_json::from_str(&json).unwrap();

        assert_eq!(raw, deserialized);
    }
}
