// octorest
// Copyright (C) SOFe
//
// Licensed under the Apache License, Version 2.0 (the License);
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an AS IS BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This file handles aliased pub re-exports.

use std::collections::HashMap;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

use super::path_entry::PathEntry;

pub enum IdItem<'a> {
    Mod(String, HashMap<String, IdItem<'a>>),
    Oper(&'a PathEntry),
}

impl<'a> IdItem<'a> {
    pub fn expect_mod(&mut self) -> &mut HashMap<String, IdItem<'a>> {
        match self {
            IdItem::Mod(_, map) => map,
            _ => panic!("Operation path collided with operation name"),
        }
    }

    fn trait_fn(&self) -> TokenStream {
        let entry = match self {
            IdItem::Oper(entry) => entry,
            _ => return quote!(),
        };
        let simple = entry.simple_name();
        let simple = Ident::new(&simple, Span::call_site());
        let method = entry.method_name();
        let method = Ident::new(&method, Span::call_site());
        let summary = &entry.operation.summary;
        let description = entry
            .operation
            .description
            .as_ref()
            .map(|s| super::remove_doc_tests(&s));
        quote! {
            #[doc = #summary]
            #[doc = ""]
            #[doc = #description]
            async fn #simple(&self) {
                crate::inner_impl::#method(self).await;
            }
        }
    }
}

impl<'a> ToTokens for IdItem<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let q = match self {
            IdItem::Mod(name, items) => {
                use heck::CamelCase;

                let delegates = items.iter().map(|(_, value)| value);
                let ident = Ident::new(name, Span::call_site());

                let trait_name = Ident::new(&name.to_camel_case(), Span::call_site());
                let item_trait_fns = items.iter().map(|(_, value)| value.trait_fn());

                quote! {
                    /// API methods in the category <em>
                    #[doc = #name]
                    /// </em>
                    ///
                    /// <em>Required feature:
                    #[doc = #name]
                    /// </em>
                    #[cfg(feature = #name)]
                    pub mod #ident { #(#delegates)* }

                    /// API client extension trait for methods related to
                    #[doc = #name]
                    ///
                    /// <em>Required feature:
                    #[doc = #name]
                    /// </em>
                    #[cfg(feature = #name)]
                    #[async_trait::async_trait]
                    pub trait #trait_name : crate::AbstractClient {
                        #(#item_trait_fns)*
                    }

                    impl<C: crate::AbstractClient> #trait_name for C {}
                }
            }
            IdItem::Oper(entry) => {
                let entry_name = entry.method_name();
                let entry_name = Ident::new(&entry_name, Span::call_site());
                let last_name = entry.simple_name();
                let last_name = Ident::new(&last_name, Span::call_site());
                quote! { pub use crate::inner_impl::#entry_name as #last_name; }
            }
        };
        q.to_tokens(tokens)
    }
}
