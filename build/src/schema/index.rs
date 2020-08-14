use std::collections::HashMap;

use getset::{Getters, MutGetters};
use serde::Deserialize;

use super::{Parameter, Schema};

#[derive(Deserialize, Getters, MutGetters)]
#[serde(rename_all = "camelCase")]
#[getset(get = "pub")]
pub struct Index {
    info: Info,
    external_docs: ExternalDocs,
    paths: Paths,
    components: Components,
}

#[derive(Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
#[getset(get = "pub")]
pub struct Info {
    version: String,
    title: String,
    description: String,
    license: License,
    terms_of_service: String,
    contact: Contact,
}

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct License {
    name: String,
}

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct Contact {
    name: Option<String>,
    email: Option<String>,
}

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct Server {
    url: String,
}

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct ExternalDocs {
    description: String,
    url: String,
}

#[derive(Deserialize)]
pub struct Paths(HashMap<String, PathItem>);

impl Paths {
    pub fn get(&self) -> &HashMap<String, PathItem> {
        &self.0
    }
}

#[derive(Deserialize)]
pub struct PathItem(HashMap<String, super::Operation>);

impl PathItem {
    pub fn get(&self) -> &HashMap<String, super::Operation> {
        &self.0
    }
}

#[derive(Deserialize)]
pub struct Components {
    schemas: HashMap<String, Schema>,
    parameters: HashMap<String, Parameter>,
}
