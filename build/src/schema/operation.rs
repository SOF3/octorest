use std::borrow::Cow;

use getset::{CopyGetters, Getters};
use serde::Deserialize;

use super::{ExternalDocs, Schema, MaybeRef};

#[derive(Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
#[getset(get = "pub")]
pub struct Operation<'sch> {
    summary: Cow<'sch, str>,
    description: Cow<'sch, str>,
    operation_id: Cow<'sch, str>,
    tags: Vec<Cow<'sch, str>,>,
    external_docs: Option<ExternalDocs<'sch>>,
    #[serde(default)]
    #[serde(borrow)]
    parameters: Vec<MaybeRef<'sch, Parameter<'sch>>>,
    request_body: Option<RequestBody<'sch>>,
    responses: Responses<'sch>,
}

#[derive(Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
pub struct Parameter<'sch> {
    #[serde(borrow)]
    #[getset(get = "pub")]
    name: Cow<'sch, str>,
    #[getset(get_copy = "pub")]
    #[serde(rename = "in")]
    location: ParameterLocation,
    #[getset(get = "pub")]
    description: Option<Cow<'sch, str>,>,
    #[getset(get = "pub")]
    schema: MaybeRef<'sch, Schema<'sch>>,
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
pub struct RequestBody<'sch> {
    #[serde(with = "tuple_vec_map")]
    #[serde(borrow)]
    content: Vec<(Cow<'sch, str>, MediaType<'sch>)>,
}

impl<'sch> RequestBody<'sch> {
    pub fn content(&self) -> impl Iterator<Item = (&str, &MediaType<'sch>)> {
        self.content.iter().map(|(k, v)| (k.as_ref(), v))
    }
}

#[derive(Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct MediaType<'sch> {
    #[serde(borrow)]
    #[getset(get = "pub")]
    schema: MaybeRef<'sch, Schema<'sch>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Responses<'sch>(
    #[serde(with = "tuple_vec_map")]
    #[serde(borrow)]
    Vec<(u16, Response<'sch>)>,
);

impl<'sch> Responses<'sch> {
    pub fn get(&self) -> impl Iterator<Item = (&u16, &Response<'sch>)> {
        self.0.iter().map(|(k, v)| (k, v))
    }
}

#[derive(Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct Response<'sch> {
    #[getset(get = "pub")]
    #[serde(borrow)]
    description: Option<Cow<'sch, str>>,
    // key is mime type, usually application/json
    #[serde(default)]
    #[serde(with = "tuple_vec_map")]
    content: Vec<(Cow<'sch, str>, MediaType<'sch>)>,
}

impl<'sch> Response<'sch> {
    pub fn content(&self) -> impl Iterator<Item = (&str, &MediaType<'sch>)> {
        self.content.iter().map(|(k, v)| (k.as_ref(), v))
    }
}
