use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use getset::{CopyGetters, Getters};
use serde::{de::IgnoredAny, Deserialize, Deserializer};

use super::MaybeRef;
use crate::gen;

#[derive(Deserialize, Getters, CopyGetters)]
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
        #[derive(Deserialize)]
        #[serde(tag = "type")]
        enum Tagged<'sch> {
            #[serde(borrow)]
            String(StringSchema<'sch>),
            Integer(IntegerSchema<'sch>),
            Number(NumberSchema),
            Boolean(BooleanSchema),
            Object(ObjectSchema<'sch>),
            Array(ArraySchema<'sch>),
        }

        #[derive(Deserialize)]
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

#[derive(Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
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
}

#[derive(Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
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
pub struct ObjectSchema<'sch> {
    #[getset(get = "pub")]
    #[serde(default)]
    #[serde(borrow)]
    required: Vec<Cow<'sch, str>>,
    #[getset(get = "pub")]
    #[serde(default)]
    properties: HashMap<Cow<'sch, str>, MaybeRef<'sch, Schema<'sch>>>,
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

#[derive(Deserialize)]
#[serde(untagged)]
#[serde(deny_unknown_fields)]
pub enum AdditionalProperties<'sch> {
    Schema(#[serde(borrow)] MaybeRef<'sch, Box<Schema<'sch>>>),
    Bool(bool),
}

#[derive(Deserialize, Getters, CopyGetters)]
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
