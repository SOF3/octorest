use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

use getset::{CopyGetters, Getters};
use serde::{de::Error, Deserialize, Deserializer};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    #[serde(flatten)]
    #[getset(get = "pub")]
    typed: Typed,
    #[getset(get = "pub")]
    description: Option<String>,
    #[serde(default)]
    #[getset(get_copy = "pub")]
    nullable: bool,
    #[serde(default)]
    #[getset(get_copy = "pub")]
    deprecated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Typed {
    String(StringSchema),
    Integer(IntegerSchema),
    Number(NumberSchema),
    Object(ObjectSchema),
    Array(ArraySchema),
    Boolean(BooleanSchema),
    Unknown,
}

impl<'de> Deserialize<'de> for Typed {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let mut value = serde_json::Map::<String, serde_json::Value>::deserialize(d)?;
        if let Some(ty) = value.remove("type") {
            if let serde_json::Value::String(str) = ty {
                let value = serde_json::Value::Object(value);
                fn me<E: Error>(err: impl fmt::Display) -> E {
                    E::custom(err)
                }
                match str.as_ref() {
                    "string" => {
                        return Ok(Typed::String(serde_json::from_value(value).map_err(me)?))
                    }
                    "integer" => {
                        return Ok(Typed::Integer(serde_json::from_value(value).map_err(me)?))
                    }
                    "number" => {
                        return Ok(Typed::Number(serde_json::from_value(value).map_err(me)?))
                    }
                    "object" => {
                        return Ok(Typed::Object(serde_json::from_value(value).map_err(me)?))
                    }
                    "array" => return Ok(Typed::Array(serde_json::from_value(value).map_err(me)?)),
                    "boolean" => {
                        return Ok(Typed::Boolean(serde_json::from_value(value).map_err(me)?))
                    }
                    _ => (),
                }
            }
        }
        Ok(Typed::Unknown)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct StringSchema {
    #[getset(get = "pub")]
    default: Option<String>,
    #[getset(get = "pub")]
    enum_: Option<Vec<String>>,
    #[getset(get = "pub")]
    #[serde(default)]
    #[allow(dead_code)]
    pattern: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct IntegerSchema {
    #[getset(get_copy = "pub")]
    default: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NumberSchema {
    #[getset(get_copy = "pub")]
    default: Option<f64>,
}

impl Eq for NumberSchema {} // only expect finite numbers here

#[allow(clippy::derive_hash_xor_eq)]
impl std::hash::Hash for NumberSchema {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {
        // we don't care about the default
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BooleanSchema {
    #[getset(get_copy = "pub")]
    default: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ObjectSchema {
    #[getset(get = "pub")]
    #[serde(default)]
    default: Option<serde_json::Map<String, serde_json::Value>>,
    #[getset(get = "pub")]
    #[serde(default)]
    required: Vec<String>,
    #[getset(get = "pub")]
    #[serde(default)]
    properties: HashMap<String, Schema>,
    #[getset(get = "pub")]
    #[serde(default)]
    additional_properties: Option<Box<Schema>>,
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for ObjectSchema {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // we don't care about the default
        self.required.hash(state);

        let mut properties = self.properties.iter().collect::<Vec<_>>();
        properties.sort_by(|a, b| a.0.cmp(b.0));
        properties.hash(state);

        self.additional_properties.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ArraySchema {
    #[getset(get = "pub")]
    #[serde(default)]
    default: Option<Vec<serde_json::Value>>,
    #[getset(get = "pub")]
    items: Box<Schema>,
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for ArraySchema {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // we don't care about the default
        self.items.hash(state);
    }
}
