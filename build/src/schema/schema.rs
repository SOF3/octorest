use std::borrow::Cow;
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use getset::{CopyGetters, Getters};
use serde::{de::IgnoredAny, Deserialize, Deserializer};

use super::MaybeRef;
use crate::gen;

#[derive(Debug, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
pub struct Schema<'sch> {
    #[serde(flatten)]
    #[serde(borrow)]
    #[getset(get = "pub")]
    typed: Typed<'sch>,
    #[getset(get = "pub")]
    title: Option<Cow<'sch, str>>,
    #[getset(get = "pub")]
    description: Option<Cow<'sch, str>>,
    #[serde(default)]
    #[getset(get_copy = "pub")]
    nullable: bool,
    #[serde(default)]
    #[getset(get_copy = "pub")]
    deprecated: bool,
    #[serde(default)]
    read_only: bool,
    #[getset(get_copy = "pub")]
    min_items: Option<usize>, // only used in ArraySchema::items

    // #[serde(skip)]
    type_def: Cell<Option<usize>>,
}

impl<'sch> Schema<'sch> {
    pub fn get_type_def<'t>(
        &self,
        types: &'t mut gen::Types<'sch>,
    ) -> Option<&'t Rc<gen::TypeDef<'sch>>> {
        match self.type_def.get() {
            Some(id) => Some(
                types
                    .defs_mut()
                    .get(id)
                    .expect("set_type_def_id was called with an invalid id"),
            ),
            None => None,
        }
    }

    pub fn set_type_def_id(&self, id: usize) {
        let old = self.type_def.replace(Some(id));
        if old.is_some() {
            panic!("Call to set_type_def_id on the same schema multiple times");
        }
    }
}

#[derive(Debug)]
pub enum Typed<'sch> {
    String(StringSchema<'sch>),
    Integer(IntegerSchema<'sch>),
    Number(NumberSchema),
    Boolean(BooleanSchema),
    Object(ObjectSchema<'sch>),
    Array(ArraySchema<'sch>),
}

impl<'sch> Typed<'sch> {
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

impl<'de: 'sch, 'sch> Deserialize<'de> for Typed<'sch> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        /*
        use serde::de::Error;

        let mut value = serde_json::Map::deserialize(d)?;
        Ok(match value.remove("type") {
            Some(serde_json::Value::String(ty)) => {
                match ty.as_str() {
                    "string" => Self::String(serde_json::from_value(serde_json::Value::Object(value))
                                             .map_err(|err| D::Error::custom(err))?),
                    "integer" => Self::Integer(serde_json::from_value(serde_json::Value::Object(value)).map_err(|err| D::Error::custom(err))?),
                    "number" => Self::Number(serde_json::from_value(serde_json::Value::Object(value)).map_err(|err| D::Error::custom(err))?),
                    "boolean" => Self::Boolean(serde_json::from_value(serde_json::Value::Object(value)).map_err(|err| D::Error::custom(err))?),
                    "object" => Self::Object(serde_json::from_value(serde_json::Value::Object(value)).map_err(|err| D::Error::custom(err))?),
                    "array" => Self::Array(serde_json::from_value(serde_json::Value::Object(value)).map_err(|err| D::Error::custom(err))?),
                    _ => return Err(D::Error::custom("Unknown type")),
                }
            },
            _ if value.get("items").is_some() => {
                Self::Array(serde_json::from_value(serde_json::Value::Object(value)).map_err(|err| D::Error::custom(err))?)
            },
            _ => Self::Object(serde_json::from_value(serde_json::Value::Object(value)).map_err(|err| D::Error::custom(err))?),
        })

        // TODO do not use serde_json::Value intermediate
        */

        #[derive(Debug, Deserialize)]
        #[serde(tag = "type")]
        #[serde(rename_all = "camelCase")]
        enum Tagged<'sch> {
            #[serde(borrow)]
            String(StringSchema<'sch>),
            Integer(IntegerSchema<'sch>),
            Number(NumberSchema),
            Boolean(BooleanSchema),
            Object(ObjectSchema<'sch>),
            Array(ArraySchema<'sch>),
        }

        #[derive(Debug, Deserialize)]
        #[serde(untagged)]
        enum Untagged<'sch> {
            #[serde(borrow)]
            Tagged(Tagged<'sch>),
            // using the `items` tag
            Array(ArraySchema<'sch>),
            Object(ObjectSchema<'sch>),
        }

        let untagged = Untagged::deserialize(d)?;

        Ok(match untagged {
            Untagged::Tagged(Tagged::String(s)) => Typed::String(s),
            Untagged::Tagged(Tagged::Integer(s)) => Typed::Integer(s),
            Untagged::Tagged(Tagged::Number(s)) => Typed::Number(s),
            Untagged::Tagged(Tagged::Boolean(s)) => Typed::Boolean(s),
            Untagged::Tagged(Tagged::Array(s)) | Untagged::Array(s) => Typed::Array(s),
            Untagged::Tagged(Tagged::Object(s)) | Untagged::Object(s) => Typed::Object(s),
        })
    }
}

#[derive(Debug, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
pub struct StringSchema<'sch> {
    #[getset(get = "pub")]
    default: Option<Cow<'sch, str>>,
    #[getset(get = "pub")]
    enum_: Option<Vec<Cow<'sch, str>>>,
    #[getset(get = "pub")]
    pattern: Option<Cow<'sch, str>>,
    #[getset(get = "pub")]
    format: Option<Cow<'sch, str>>,
    #[getset(get_copy = "pub")]
    min_length: Option<usize>,
    #[getset(get_copy = "pub")]
    max_length: Option<usize>,
    example: Option<IgnoredAny>,
}

#[derive(Debug, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
pub struct IntegerSchema<'sch> {
    #[getset(get_copy = "pub")]
    default: Option<i64>,
    #[getset(get = "pub")]
    format: Option<Cow<'sch, str>>,
    #[getset(get_copy = "pub")]
    maximum: Option<i64>,
    #[getset(get_copy = "pub")]
    minimum: Option<i64>,
}

#[derive(Debug, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
pub struct NumberSchema {
    #[getset(get_copy = "pub")]
    default: Option<f64>,
}

#[derive(Debug, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
pub struct BooleanSchema {
    #[getset(get_copy = "pub")]
    default: Option<bool>,
}

#[derive(Debug, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
pub struct ObjectSchema<'sch> {
    #[getset(get = "pub")]
    #[serde(default)]
    #[serde(borrow)]
    required: BTreeSet<Cow<'sch, str>>,
    #[getset(get = "pub")]
    #[serde(default)]
    properties: BTreeMap<Cow<'sch, str>, MaybeRef<'sch, Schema<'sch>>>,
    #[getset(get = "pub")]
    additional_properties: Option<AdditionalProperties<'sch>>,
    #[getset(get_copy = "pub")]
    max_properties: Option<usize>,
    #[getset(get = "pub")]
    any_of: Option<Vec<MaybeRef<'sch, Schema<'sch>>>>,
    #[getset(get = "pub")]
    one_of: Option<Vec<MaybeRef<'sch, Schema<'sch>>>>,
    #[getset(get = "pub")]
    all_of: Option<Vec<MaybeRef<'sch, Schema<'sch>>>>,
    #[getset(get_copy = "pub")]
    default: Option<IgnoredAny>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum AdditionalProperties<'sch> {
    Schema(#[serde(borrow)] MaybeRef<'sch, Box<Schema<'sch>>>),
    Bool(bool),
}

#[derive(Debug, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
pub struct ArraySchema<'sch> {
    #[getset(get = "pub")]
    #[serde(borrow)]
    items: MaybeRef<'sch, Box<Schema<'sch>>>,
    #[getset(get_copy = "pub")]
    min_items: Option<usize>,
    // unused, probably documentation bug
    default: Option<IgnoredAny>,
}
