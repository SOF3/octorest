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
pub struct PathItem(HashMap<String, MaybeRef<super::Operation>>);

impl PathItem {
    pub fn get(&self) -> &HashMap<String, MaybeRef<super::Operation>> {
        &self.0
    }
    pub(super) fn get_mut(&mut self) -> &mut HashMap<String, MaybeRef<super::Operation>> {
        &mut self.0
    }
}

pub enum MaybeRef<T> {
    Value(T),
    Ref(Ref),
}

impl<T> MaybeRef<T> {
    pub fn get(&self) -> &T {
        match self {
            Self::Value(t) => t,
            Self::Ref(_) => panic!("Call to MaybeRef::get() before ref resolution"),
        }
    }
}

impl<'de, T> Deserialize<'de> for MaybeRef<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(Self::Ref(Ref::deserialize(d)?))
    }
}

#[derive(Deserialize, Getters)]
pub struct Ref {
    #[serde(rename = "$ref")]
    #[get = "pub(super)"]
    ref_path: PathBuf,
}
