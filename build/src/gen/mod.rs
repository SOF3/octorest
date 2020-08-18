use proc_macro2::TokenStream;
use quote::quote;

use self::from_schema::schema_to_def;
pub use self::tree::{NameComponent, NameComponents};
use self::tree::{NameTree, NameTreeResolve, TreeHandle};
use self::types::Lifetime;
pub use self::types::{TypeDef, Types};
use crate::{idents, schema};

mod from_schema;
mod tree;
mod types;

pub fn gen<'sch>(index: &'sch schema::Index<'sch>) -> TokenStream {
    let mut types = Types::default();

    crate::task("Generate types for .components.schemas", || {
        for (name, schema) in index.components().schemas() {
            schema_to_def(
                &mut types,
                index,
                schema,
                vec![
                    NameComponent::prepend(&**name),
                    NameComponent::append("schema"),
                    NameComponent::append("comp"),
                ],
            );
        }
    });
    crate::task("Generate types for .components.parameters", || {
        for (name, param) in index.components().parameters() {
            let (schema, name_comps) =
                index
                    .components()
                    .resolve_schema(param.schema(), crate::id, || {
                        vec![
                            NameComponent::prepend(&**name),
                            NameComponent::append("param"),
                            NameComponent::append("comp"),
                        ]
                    });
            schema_to_def(&mut types, index, schema, name_comps);
        }
    });
    crate::task("Generate types for .components.headers", || {
        for (name, media_type) in index.components().headers() {
            let (schema, name_comps) =
                index
                    .components()
                    .resolve_schema(media_type.schema(), crate::id, || {
                        vec![
                            NameComponent::prepend(&**name),
                            NameComponent::append("header"),
                            NameComponent::append("comp"),
                        ]
                    });
            schema_to_def(&mut types, index, schema, name_comps);
        }
    });
    crate::task("Generate types for .components.responses", || {
        for (name, response) in index.components().responses() {
            for (mime, media_type) in response.content() {
                let (schema, name_comps) =
                    index
                        .components()
                        .resolve_schema(media_type.schema(), crate::id, || {
                            vec![
                                NameComponent::prepend(&**name),
                                NameComponent::append("response"),
                                NameComponent::append("comp"),
                            ]
                        });
                schema_to_def(&mut types, index, schema, name_comps);
            }
        }
    });

    let types = types.finalize();

    let mut tag_getters = quote!();
    let mut tag_structs = quote!();
    for tag in index.tags() {
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
        #[allow(unused_parens)]
        #[allow(clippy::double_parens)]
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
        #[allow(unused_parens)]
        #[allow(clippy::double_parens)]
        pub mod types {
            #types
        }
    };

    // dbg!(ret.to_string());

    ret
}
