//! The JSON tree a request document is read as, and the one parser that
//! builds it (milestone 5e1 §3.1).
//!
//! Objects keep their members in document order, so the checks in `request`
//! can name the JSON path of the first problem they find (§3.4 and §3.5).
//! `parse_json` refuses the three things that would otherwise be a silent
//! repair of the request's text: a repeated object key, which serde_json's
//! own map would resolve by keeping the last value; a JSON syntax error; and
//! trailing text after the one document the schema describes (§2). A `null`
//! is kept as a value of its own, because `null` is never a stand-in for an
//! absent field (§2). Only the document's shape is checked here: identifiers,
//! residue letters, coordinates, completeness, genetic code and numbering
//! carry their biological meaning, so they are kept as given and core reports
//! them (§1, §3.5).
//!
//! # The rules this module enforces
//!
//! - duplicate object keys are refused, naming the key and its place;
//! - a JSON syntax error, a duplicate key and trailing text are reported with
//!   serde_json's own message, which ends with ` at line L column C`;
//! - an integer is a JSON number `serde_json::Number::as_i64` accepts, so a
//!   fraction, an exponent form or a value outside `i64` is not one (§3.4);
//! - a number is any JSON number read with `as_f64`, and a non-finite value
//!   is an error (§3.4).
//!
//! A `Json` value is built only by `parse_json`; nothing here interprets the
//! request's biology.

use serde::de::Deserialize;
use serde::de::Deserializer;
use serde::de::Error;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;
use serde_json::Deserializer as JsonDeserializer;
use serde_json::Number;
use std::fmt::Formatter;

/// A JSON value whose objects keep their members in document order. Built
/// only by `parse_json`, which refuses duplicate keys.
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl<'de> Deserialize<'de> for Json {
    fn deserialize<D>(deserializer: D) -> Result<Json, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct JsonVisitor;

        impl<'de> Visitor<'de> for JsonVisitor {
            type Value = Json;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a JSON value")
            }

            /// JSON's `null`, kept as a value rather than as an absent
            /// field.
            fn visit_unit<E>(self) -> Result<Json, E>
            where
                E: Error,
            {
                Ok(Json::Null)
            }

            fn visit_bool<E>(self, value: bool) -> Result<Json, E>
            where
                E: Error,
            {
                Ok(Json::Bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Json, E>
            where
                E: Error,
            {
                Ok(Json::Number(Number::from(value)))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Json, E>
            where
                E: Error,
            {
                Ok(Json::Number(Number::from(value)))
            }

            /// A number that is not finite has no JSON spelling, so it is
            /// refused rather than stored as something else.
            fn visit_f64<E>(self, value: f64) -> Result<Json, E>
            where
                E: Error,
            {
                match Number::from_f64(value) {
                    None => Err(Error::custom("expected a finite number")),
                    Some(number) => Ok(Json::Number(number)),
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<Json, E>
            where
                E: Error,
            {
                Ok(Json::String(value.to_string()))
            }

            fn visit_string<E>(self, value: String) -> Result<Json, E>
            where
                E: Error,
            {
                Ok(Json::String(value))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Json, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut items: Vec<Json> = Vec::new();
                loop {
                    match seq.next_element::<Json>() {
                        Ok(Some(item)) => items.push(item),
                        Ok(None) => break,
                        Err(error) => return Err(error),
                    }
                }
                Ok(Json::Array(items))
            }

            /// Members are kept in document order, and a key already seen is
            /// refused rather than resolved by keeping one of the values.
            fn visit_map<A>(self, mut map: A) -> Result<Json, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut members: Vec<(String, Json)> = Vec::new();
                loop {
                    let key = map.next_key::<String>()?;
                    match key {
                        None => break,
                        Some(key) => {
                            if members.iter().any(|(seen, _)| *seen == key) {
                                return Err(Error::custom(format!("duplicate key '{}'", key)));
                            }
                            let value = map.next_value::<Json>()?;
                            members.push((key, value))
                        }
                    }
                }
                Ok(Json::Object(members))
            }
        }

        deserializer.deserialize_any(JsonVisitor)
    }
}

/// Parse one JSON document. The error text is serde_json's, which ends with
/// ` at line L column C`.
pub fn parse_json(text: &str) -> Result<Json, String> {
    let mut deserializer = JsonDeserializer::from_str(text);
    let value = Json::deserialize(&mut deserializer).map_err(|error| error.to_string())?;
    deserializer.end().map_err(|error| error.to_string())?;
    Ok(value)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn members_keep_their_document_order() {
        let parsed = parse_json(r#"{"b": 1, "a": [true, null, "x", 2.5]}"#).unwrap();
        match parsed {
            Json::Object(members) => {
                let keys: Vec<&str> = members.iter().map(|(key, _)| key.as_str()).collect();
                assert_eq!(keys, vec!["b", "a"]);
                assert_eq!(
                    members[1].1,
                    Json::Array(vec![
                        Json::Bool(true),
                        Json::Null,
                        Json::String("x".to_string()),
                        Json::Number(serde_json::Number::from_f64(2.5).unwrap()),
                    ])
                );
            }
            other => panic!("expected an object, got {:?}", other),
        }
    }

    #[test]
    fn a_duplicate_key_is_refused_with_its_place() {
        let error = parse_json("{\"a\": 1,\n \"a\": 2}").unwrap_err();
        assert!(error.starts_with("duplicate key 'a'"), "{}", error);
        assert!(error.contains("line 2"), "{}", error);
        let nested = parse_json(r#"{"x": {"k": 1, "k": 1}}"#).unwrap_err();
        assert!(nested.starts_with("duplicate key 'k'"), "{}", nested);
    }

    #[test]
    fn trailing_text_and_bad_syntax_are_refused() {
        assert!(parse_json("{} {}")
            .unwrap_err()
            .starts_with("trailing characters"));
        assert!(parse_json("").is_err());
        assert!(parse_json("{\"a\": 1,}").is_err());
    }
}
