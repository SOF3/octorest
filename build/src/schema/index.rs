use std::borrow::Cow;
use std::collections::BTreeMap;

use getset::{Getters, MutGetters};
use serde::de::IgnoredAny;
use serde::Deserialize;

use super::{MaybeRef, MediaType, Parameter, Ref, Response, Schema};

#[derive(Deserialize, Getters, MutGetters)]
#[serde(rename_all = "camelCase")]
#[getset(get = "pub")]
pub struct Index<'sch> {
    #[serde(borrow)]
    info: Info<'sch>,
    external_docs: ExternalDocs<'sch>,
    paths: Paths<'sch>,
    components: Components<'sch>,
}

#[derive(Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
#[getset(get = "pub")]
pub struct Info<'sch> {
    version: Cow<'sch, str>,
    title: Cow<'sch, str>,
    description: Cow<'sch, str>,
    license: License<'sch>,
    terms_of_service: Cow<'sch, str>,
    contact: Contact<'sch>,
}

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct License<'sch> {
    name: Cow<'sch, str>,
}

#[derive(Deserialize, Getters)]
pub struct Contact<'sch> {
    #[getset(get_copy = "pub")]
    name: Option<Cow<'sch, str>>,
    #[getset(get_copy = "pub")]
    email: Option<Cow<'sch, str>>,
}

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct Server<'sch> {
    url: Cow<'sch, str>,
}

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct ExternalDocs<'sch> {
    description: Cow<'sch, str>,
    url: Cow<'sch, str>,
}

#[derive(Deserialize)]
pub struct Paths<'sch>(
    #[serde(with = "tuple_vec_map")]
    #[serde(borrow)]
    Vec<(Cow<'sch, str>, PathItem<'sch>)>,
);

impl<'sch> Paths<'sch> {
    pub fn get(&self) -> impl Iterator<Item = (&str, &PathItem<'sch>)> {
        self.0.iter().map(|(k, v)| (&**k, v))
    }
}

#[derive(Deserialize)]
pub struct PathItem<'sch>(
    #[serde(with = "tuple_vec_map")]
    #[serde(borrow)]
    Vec<(Cow<'sch, str>, super::Operation<'sch>)>,
);

impl<'sch> PathItem<'sch> {
    pub fn get(&self) -> impl Iterator<Item = (&str, &super::Operation<'sch>)> {
        self.0.iter().map(|(k, v)| (&**k, v))
    }
}

#[derive(Deserialize, Getters)]
#[serde(deny_unknown_fields)]
pub struct Components<'sch> {
    #[serde(borrow)]
    #[getset(get = "pub")]
    parameters: BTreeMap<Cow<'sch, str>, Parameter<'sch>>,
    #[getset(get = "pub")]
    schemas: BTreeMap<Cow<'sch, str>, Schema<'sch>>,
    #[getset(get = "pub")]
    examples: IgnoredAny,
    #[getset(get = "pub")]
    headers: BTreeMap<Cow<'sch, str>, MediaType<'sch>>,
    #[getset(get = "pub")]
    responses: BTreeMap<Cow<'sch, str>, Response<'sch>>,
}

impl<'sch> Components<'sch> {
    pub fn resolve_schema<'t, S, F>(&'t self, mr: &'t MaybeRef<'sch, S>, f: F) -> &'t Schema<'sch>
    where
        F: FnOnce(&'t S) -> &'t Schema<'sch>,
    {
        match mr {
            MaybeRef::Owned(schema) => f(schema),
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
