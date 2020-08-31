use proc_macro2::TokenStream;
use quote::quote;

pub use self::tree::{NameComponent, NameComponents};
use self::tree::{NameTree, NameTreeResolve, TreeHandle};
use self::types::Lifetime;
pub use self::types::{TypeDef, Types};
use crate::{idents, schema};

mod from_schema;
mod operation;
mod tree;
mod types;

pub fn gen<'sch>(index: &'sch schema::Index<'sch>) -> TokenStream {
    let mut types = Types::default();

    let mut tag_getters = quote!();
    let mut tag_structs = Vec::new();
    for tag in index.tags() {
        crate::task(&format!("Generating definitions for tag {}", tag.name()), || {
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

            let mut operations: Vec<_> = Vec::new();
            for (path, path_item) in index.paths().get() {
                for (method, operation) in path_item.get() {
                    if operation.tags().contains(tag.name()) {
                        operations.push(operation::compute(path, method, operation, &mut types));
                    }
                }
            }

            tag_structs.push(move |ntr| {
                let operations = operations.iter().map(|oper| oper(ntr));
                quote! {
                    #[cfg(feature = #feature)]
                    #[cfg_attr(feature = "internal-docsrs", doc(cfg(feature = #feature)))]
                    #[doc = #doc]
                    pub struct #pascal<'c>(&'c crate::Client);

                    #[cfg(feature = #feature)]
                    impl<'c> #pascal<'c> {
                        #(#operations)*
                    }
                }
            });
        });
    }

    let (ntr, types) = types.finalize();

    let tag_structs = tag_structs.iter().map(|f| f(&ntr));
    let ret = quote! {
        /// Categorized GitHub API endpoints
        #[allow(unused_parens)]
        #[allow(clippy::double_parens)]
        pub mod apis {
            impl crate::Client {
                #tag_getters
            }

            #(#tag_structs)*
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
