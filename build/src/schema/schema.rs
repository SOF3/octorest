use std::collections::HashMap;
use std::fmt;

use getset::{CopyGetters, Getters};
use serde::{de::Error, Deserialize, Deserializer};

use super::MaybeRef;

#[derive(Debug, Clone, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    #[serde(flatten)]
    #[getset(get = "pub")]
    typed: Typed,
    #[getset(get = "pub")]
    title: Option<String>,
    #[getset(get = "pub")]
    description: Option<String>,
    #[serde(default)]
    #[getset(get_copy = "pub")]
    nullable: bool,
    #[serde(default)]
    #[getset(get_copy = "pub")]
    deprecated: bool,
    #[getset(get = "pub")]
    example: Option<serde_json::Value>,
    #[serde(default)]
    read_only: bool,
    #[getset(get_copy = "pub")]
    min_items: Option<usize>, // only used in ArraySchema::items
}

#[derive(Debug, Clone)]
pub enum Typed {
    String(StringSchema),
    Integer(IntegerSchema),
    Number(NumberSchema),
    Boolean(BooleanSchema),
    Object(ObjectSchema),
    Array(ArraySchema),
}

impl Typed {
    pub fn has_default(&self) -> bool {
        match self {
            Self::String(s) => s.default.is_some(),
            Self::Integer(s) => s.default.is_some(),
            Self::Number(s) => s.default.is_some(),
            Self::Boolean(s) => s.default.is_some(),
            Self::Object(s) => s.default.is_some(),
            Self::Array(s) => s.default.is_some(),
        }
    }
}

impl<'de> Deserialize<'de> for Typed {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let mut value = serde_json::Map::<String, serde_json::Value>::deserialize(d)?;

        fn me<E: Error>(err: impl fmt::Display) -> E {
            E::custom(err)
        }

        if let Some(ty) = value.remove("type") {
            if let serde_json::Value::String(str) = ty {
                let value = serde_json::Value::Object(value);
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
                    _ => panic!("unknown type: {:?}", value),
                }
            }
        }

        if value.get("items").is_some() {
            return Ok(Typed::Array(
                serde_json::from_value(value.into()).map_err(me)?,
            ));
        }

        // assume object
        Ok(Typed::Object(
            serde_json::from_value(value.into()).map_err(me)?,
        ))
    }
}

#[derive(Debug, Clone, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct StringSchema {
    #[getset(get = "pub")]
    default: Option<String>,
    #[getset(get = "pub")]
    enum_: Option<Vec<String>>,
    #[getset(get = "pub")]
    pattern: Option<String>,
    #[getset(get = "pub")]
    format: Option<String>,
    #[getset(get_copy = "pub")]
    min_length: Option<usize>,
    #[getset(get_copy = "pub")]
    max_length: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct IntegerSchema {
    #[getset(get_copy = "pub")]
    default: Option<i64>,
    #[getset(get = "pub")]
    format: Option<String>,
    #[getset(get_copy = "pub")]
    maximum: Option<i64>,
    #[getset(get_copy = "pub")]
    minimum: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NumberSchema {
    #[getset(get_copy = "pub")]
    default: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BooleanSchema {
    #[getset(get_copy = "pub")]
    default: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ObjectSchema {
    #[getset(get = "pub")]
    default: Option<serde_json::Map<String, serde_json::Value>>,
    #[getset(get = "pub")]
    #[serde(default)]
    required: Vec<String>,
    #[getset(get = "pub")]
    #[serde(default)]
    properties: HashMap<String, MaybeRef<Schema>>,
    #[getset(get = "pub")]
    additional_properties: Option<AdditionalProperties>,
    #[getset(get_copy = "pub")]
    max_properties: Option<usize>,
    #[getset(get = "pub")]
    any_of: Option<Vec<MaybeRef<Schema>>>,
    #[getset(get = "pub")]
    one_of: Option<Vec<MaybeRef<Schema>>>,
    #[getset(get = "pub")]
    all_of: Option<Vec<MaybeRef<Schema>>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
#[serde(deny_unknown_fields)]
pub enum AdditionalProperties {
    Schema(MaybeRef<Box<Schema>>),
    Bool(bool),
}

#[derive(Debug, Clone, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ArraySchema {
    #[getset(get = "pub")]
    default: Option<Vec<serde_json::Value>>,
    #[getset(get = "pub")]
    items: MaybeRef<Box<Schema>>,
    #[getset(get_copy = "pub")]
    min_items: Option<usize>,
    // unused, probably documentation bug
    #[getset(get = "pub")]
    required: Option<serde_json::Value>,
}
