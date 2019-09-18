// octorest
// Copyright (C) SOFe
//
// Licensed under the Apache License, Version 2.0 (the License);
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an AS IS BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;

use serde::{Deserialize, Deserializer};

type Url = String; // TODO parse and validate

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Index {
    pub openapi: SemverString,
    pub info: Info,
    pub servers: Vec<Server>,
    pub paths: HashMap<String, PathItem>,
    pub external_docs: Option<ExternalDocs>,
}

#[derive(Debug)]
pub struct SemverString(pub semver::Version);

impl<'de> Deserialize<'de> for SemverString {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        use serde::de::Error;

        let string = String::deserialize(de)?;
        let version = semver::Version::parse(&string).map_err(|err| D::Error::custom(err))?;
        Ok(Self(version))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub title: String,
    pub description: Option<String>,
    pub terms_of_service: Option<Url>,
    pub contact: Contact,
    pub license: Option<License>,
    pub version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub name: Option<String>,
    pub url: Option<Url>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct License {
    pub name: String,
    pub url: Option<Url>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub url: String,
    pub description: Option<String>,
    #[serde(default)]
    pub variables: HashMap<String, ServerVariable>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerVariable {
    #[serde(rename = "enum")]
    pub enumeration: Vec<String>,
    pub default: String,
    pub description: Option<String>,
}

pub type PathItem = HashMap<String, Operation>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    #[serde(default)]
    pub tags: Vec<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub external_docs: Option<ExternalDocs>,
    pub operation_id: String,
    pub parameters: Vec<Parameter>,
    pub request_body: Option<RequestBody>,
    pub responses: HashMap<u16, Response>, // u16 based on reqwest conventions
    #[serde(default)]
    pub deprecated: bool,
    #[serde(default)]
    pub servers: Vec<Server>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: ParameterLocation,
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub deprecated: bool,
    #[serde(default)]
    pub allow_empty_value: bool,
    // pub schema: Schema,
    // not documenting example(s) because we won't use them
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ParameterLocation {
    Query,
    Header,
    Path,
    Cookie,
}

// #[derive(Debug, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Schema {
// }

// pub type Schema = serde_json::Value; // TODO

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestBody {
    pub description: Option<String>,
    pub content: HashMap<String, MediaType>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub description: String,
    #[serde(default)]
    pub headers: HashMap<String, Header>,
    #[serde(default)]
    pub content: HashMap<String, MediaType>,
    #[serde(default)]
    pub links: HashMap<String, Link>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub deprecated: bool,
    #[serde(default)]
    pub allow_empty_value: bool,
    // pub schema: Schema,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaType {
    // pub schema: Schema,
}

pub type Link = serde_json::Value; // TODO

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDocs {
    pub description: Option<String>,
    pub url: Url,
}
