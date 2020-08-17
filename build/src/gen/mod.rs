use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::quote;

use self::from_schema::schema_to_def;
use self::tree::{NameComponent, NameTree, NameTreeResolve, TreeHandle};
use self::types::Lifetime;
pub use self::types::{TypeDef, Types};
use crate::{idents, schema};

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

    let mut tag_getters = quote!();
    let mut tag_structs = quote!();
    for tag in index.tags(){
        use heck::KebabCase;

        let feature = format!("gh-{}", tag.name().to_kebab_case());
        let snake = idents::snake(tag.name());
        let pascal = idents::pascal(tag.name());
        let doc = tag.description();

        tag_getters.extend(quote! {
            #[cfg(feature = #feature)]
            #[cfg_attr(feature = "internal-docsrs", doc(cfg(feature = #feature)))]
            #[doc = #doc]
            pub fn #snake(&self) -> #pascal<'_> {
                #pascal(self)
            }
        });

        tag_structs.extend(quote! {
            #[cfg(feature = #feature)]
            #[cfg_attr(feature = "internal-docsrs", doc(cfg(feature = #feature)))]
            #[doc = #doc]
            pub struct #pascal<'c>(&'c crate::Client);

            #[cfg(feature = #feature)]
            impl<'c> #pascal<'c> {
            }
        });
    }

    let ret = quote! {
        /// Categorized GitHub API endpoints
        pub mod apis {
            impl crate::Client {
                #tag_getters
            }

            #tag_structs
        }

        /// Miscellaneous data types used in the GitHub API.
        ///
        /// The API is designed such that users shall not need to explicitly import types from this
        /// module.
        pub mod types {
            #types
        }
    };

    // dbg!(ret.to_string());

    ret
}
