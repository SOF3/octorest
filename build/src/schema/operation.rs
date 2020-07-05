use std::collections::HashMap;

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
    external_docs: ExternalDocs,
    parameters: Vec<Parameter>,
    request_body: Option<RequestBody>,
    responses: Responses,
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
    schema: Schema,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ParameterLocation {
    Query,
    Header,
    Path,
    Cookie,
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
    schema: Schema,
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
    description: String,
    #[serde(default)]
    content: HashMap<String, MediaType>,
}
