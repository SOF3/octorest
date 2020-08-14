use std::collections::HashMap;
use std::fmt;

use getset::{CopyGetters, Getters};
use serde::Deserialize;

use super::{ExternalDocs, Schema};

#[derive(Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
#[getset(get = "pub")]
pub struct Operation {
    summary: String,
    description: String,
    operation_id: String,
    tags: Vec<String>,
    #[serde(default)]
    external_docs: Option<ExternalDocs>,
    #[serde(default)]
    parameters: Vec<MaybeRef<Parameter>>,
    request_body: Option<RequestBody>,
    responses: Responses,
}

#[derive(Debug, Clone)]
pub enum MaybeRef<T> {
    Ref(Ref),
    Owned(T),
}

impl<'de, T: for<'t> Deserialize<'t>> Deserialize<'de> for MaybeRef<T> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(d)?;
        if let serde_json::Value::Object(map) = &value {
            if let Some(serde_json::Value::String(target)) = map.get("$ref") {
                return Ok(Self::Ref(Ref {
                    target: target.to_string(),
                }));
            }
        }

        fn me<E: serde::de::Error>(err: impl fmt::Display) -> E {
            E::custom(err)
        }
        let t: T = serde_json::from_value(value).map_err(me)?;
        Ok(Self::Owned(t))
    }
}

#[derive(Debug, Clone, Getters, Deserialize)]
#[getset(get = "pub")]
pub struct Ref {
    #[serde(rename = "$ref")]
    pub target: String,
}

#[derive(Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    #[getset(get = "pub")]
    name: String,
    #[getset(get_copy = "pub")]
    #[serde(rename = "in")]
    location: ParameterLocation,
    #[getset(get = "pub")]
    description: Option<String>,
    #[getset(get = "pub")]
    schema: MaybeRef<Schema>,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ParameterLocation {
    Query,
    Header,
    Path,
}

#[derive(Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
#[getset(get = "pub")]
pub struct RequestBody {
    content: HashMap<String, MediaType>,
}

#[derive(Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
#[getset(get = "pub")]
pub struct MediaType {
    schema: MaybeRef<Schema>,
    example: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Responses(HashMap<u16, Response>);

impl Responses {
    pub fn get(&self) -> impl Iterator<Item = (&u16, &Response)> {
        self.0.iter()
    }
}

#[derive(Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
#[getset(get = "pub")]
pub struct Response {
    description: Option<String>,
    // key is mime type, usually application/json
    #[serde(default)]
    content: HashMap<String, MediaType>,
}
