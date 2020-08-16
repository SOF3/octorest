use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::quote;

use self::from_schema::schema_to_def;
use self::tree::{NameComponent, NameTree, NameTreeResolve, TreeHandle};
pub use self::types::TypeDef;
use self::types::{Lifetime, Types};
use crate::schema;

mod from_schema;
mod tree;
mod types;

pub fn gen(index: &schema::Index<'_>) -> TokenStream {
    let mut types = Types::default();

    crate::task("Generate types for .components.schemas", || {
        for (name, schema) in index.components().schemas() {
            schema_to_def(
                &mut types,
                index,
                schema,
                [&**name, "schema", "comp", ""]
                    .iter()
                    .map(|name| Cow::Borrowed(*name)),
            );
        }
    });
    crate::task("Generate types for .components.parameters", || {
        for (name, param) in index.components().parameters() {
            schema_to_def(
                &mut types,
                index,
                index.components().resolve_schema(param.schema(), crate::id),
                [&**name, "param", "comp", ""]
                    .iter()
                    .map(|name| Cow::Borrowed(*name)),
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
                [&**name, "header", "comp", ""]
                    .iter()
                    .map(|name| Cow::Borrowed(*name)),
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
                    [&**name, "response", "comp", ""]
                        .iter()
                        .map(|name| Cow::Borrowed(*name)),
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

fn iter_change_first<T>(
    mut iter: impl Iterator<Item = T>,
    change: impl FnOnce(T) -> T,
) -> impl Iterator<Item = T> {
    iter.next()
        .map(move |first| change(first))
        .into_iter()
        .chain(iter)
}
