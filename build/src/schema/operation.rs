use std::collections::HashMap;

use getset::Getters;
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

#[derive(Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
#[getset(get = "pub")]
pub struct Parameter {
    name: String,
    #[serde(rename = "in")]
    location: ParameterLocation,
    description: Option<String>,
    schema: Schema,
}

#[derive(Deserialize)]
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
    pub fn get(&self) -> &HashMap<u16, Response> {
        &self.0
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
