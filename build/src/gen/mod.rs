use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::quote;

use self::from_schema::schema_to_def;
use self::tree::{NameComponent, NameTree, NameTreeResolve, TreeHandle};
use self::types::Lifetime;
pub use self::types::{TypeDef, Types};
use crate::schema;

mod from_schema;
mod tree;
mod types;

macro_rules! cow_iter {
    ($size:literal : $($args:expr),* $(,)?) => {{
        vec![$(Cow::from($args)),*]
    }};
}

pub fn gen<'sch>(index: &'sch schema::Index<'sch>) -> TokenStream {
    let mut types = Types::default();

    crate::task("Generate types for .components.schemas", || {
        for (name, schema) in index.components().schemas() {
            schema_to_def(
                &mut types,
                index,
                schema,
                cow_iter![3: &**name, "schema", "comp"],
            );
        }
    });
    crate::task("Generate types for .components.parameters", || {
        for (name, param) in index.components().parameters() {
            schema_to_def(
                &mut types,
                index,
                index.components().resolve_schema(param.schema(), crate::id),
                cow_iter![3: &**name, "param", "comp"],
            );
        }
    });
    crate::task("Generate types for .components.headers", || {
        for (name, media_type) in index.components().headers() {
            schema_to_def(
                &mut types,
                index,
                index
                    .components()
                    .resolve_schema(media_type.schema(), crate::id),
                cow_iter![3: &**name, "header", "comp"],
            );
        }
    });
    crate::task("Generate types for .components.responses", || {
        for (name, response) in index.components().responses() {
            for (mime, media_type) in response.content() {
                schema_to_def(
                    &mut types,
                    index,
                    index
                        .components()
                        .resolve_schema(media_type.schema(), crate::id),
                    cow_iter![3: &**name, "response", "comp"],
                );
            }
        }
    });

    let types = types.finalize();

    let ret = quote! {
        pub mod types {
            #types
        }
    };

    dbg!(ret.to_string());
    ret
}
