use std::collections::HashMap;
use std::fmt;

use getset::{CopyGetters, Getters};
use serde::{de::Error, Deserialize, Deserializer};

#[derive(Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    #[serde(flatten)]
    typed: TypedSchema,
    #[getset(get = "pub")]
    description: Option<String>,
    #[serde(default)]
    nullable: bool,
    #[serde(default)]
    deprecated: bool,
}

pub enum TypedSchema {
    String(StringSchema),
    Integer(IntegerSchema),
    Number(NumberSchema),
    Object(ObjectSchema),
    Array(ArraySchema),
    Boolean(BooleanSchema),
    Unknown,
}

impl<'de> Deserialize<'de> for TypedSchema {
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
                        return Ok(TypedSchema::String(
                            serde_json::from_value(value).map_err(me)?,
                        ))
                    }
                    "integer" => {
                        return Ok(TypedSchema::Integer(
                            serde_json::from_value(value).map_err(me)?,
                        ))
                    }
                    "number" => {
                        return Ok(TypedSchema::Number(
                            serde_json::from_value(value).map_err(me)?,
                        ))
                    }
                    "object" => {
                        return Ok(TypedSchema::Object(
                            serde_json::from_value(value).map_err(me)?,
                        ))
                    }
                    "array" => {
                        return Ok(TypedSchema::Array(
                            serde_json::from_value(value).map_err(me)?,
                        ))
                    }
                    "boolean" => {
                        return Ok(TypedSchema::Boolean(
                            serde_json::from_value(value).map_err(me)?,
                        ))
                    }
                    _ => (),
                }
            }
        }
        Ok(TypedSchema::Unknown)
    }
}

#[derive(Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct StringSchema {
    #[getset(get = "pub")]
    default: Option<String>,
    #[getset(get = "pub")]
    enum_: Option<Vec<String>>,
    #[getset(get = "pub")]
    #[serde(default)]
    pattern: Option<String>,
}

#[derive(Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct IntegerSchema {
    #[getset(get_copy = "pub")]
    default: Option<i64>,
}

#[derive(Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NumberSchema {
    #[getset(get_copy = "pub")]
    default: Option<f64>,
}

#[derive(Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BooleanSchema {
    #[getset(get_copy = "pub")]
    default: Option<bool>,
}

#[derive(Deserialize, Getters, CopyGetters)]
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

#[derive(Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ArraySchema {
    #[getset(get = "pub")]
    #[serde(default)]
    default: Option<Vec<serde_json::Value>>,
    #[getset(get = "pub")]
    items: Box<Schema>,
}
