use std::collections::HashMap;
use std::path::PathBuf;

use getset::{Getters, MutGetters};
use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Getters, MutGetters)]
#[serde(rename_all = "camelCase")]
#[getset(get = "pub")]
pub struct Index {
    info: Info,
    external_docs: ExternalDocs,
    #[getset(get_mut = "pub(super)")]
    paths: Paths,
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
    name: String,
    email: String,
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
    pub(super) fn get_mut(&mut self) -> &mut HashMap<String, PathItem> {
        &mut self.0
    }
}

#[derive(Deserialize)]
pub struct PathItem(HashMap<String, super::Operation>);

impl PathItem {
    pub fn get(&self) -> &HashMap<String, super::Operation> {
        &self.0
    }
    pub(super) fn get_mut(&mut self) -> &mut HashMap<String, super::Operation> {
        &mut self.0
    }
}
