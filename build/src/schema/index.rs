use std::collections::HashMap;

use getset::{Getters, MutGetters};
use serde::Deserialize;

use super::{MaybeRef, MediaType, Parameter, Ref, Response, Schema};

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

#[derive(Deserialize, Getters)]
#[serde(deny_unknown_fields)]
pub struct Components {
    #[getset(get = "pub")]
    parameters: HashMap<String, Parameter>,
    #[getset(get = "pub")]
    schemas: HashMap<String, Schema>,
    #[getset(get = "pub")]
    examples: HashMap<String, serde_json::Value>, // unused
    #[getset(get = "pub")]
    headers: HashMap<String, MediaType>,
    #[getset(get = "pub")]
    responses: HashMap<String, Response>,
}

impl Components {
    pub fn resolve_schema<'t>(&'t self, mr: &'t MaybeRef<Schema>) -> &'t Schema {
        match mr {
            MaybeRef::Owned(schema) => schema,
            MaybeRef::Ref(Ref { target }) => {
                if let Some(name) = target.strip_prefix("#/components/schemas/") {
                    match self.schemas.get(name) {
                        Some(schema) => schema,
                        None => panic!("Schema {:?} not found", target),
                    }
                } else {
                    panic!("Schema {:?} not found", target)
                }
            }
        }
    }
}
