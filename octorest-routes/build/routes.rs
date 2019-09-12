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

use std::collections::HashMap;
use std::io::Read;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::io::Write;

use crate::result::*;
use crate::schema;

pub struct RouteStore {
    info: schema::Info,
    server: String,
    external_docs: schema::ExternalDocs,
    paths: Vec<PathEntry>,
}

impl RouteStore {
    pub fn try_new<R: Read>(json: R) -> Result<RouteStore> {
        let index: schema::Index = serde_json::from_reader(json)?;

        Ok(RouteStore {
            info: index.info,
            server: index.servers[0].url.clone(),
            external_docs: index.external_docs.unwrap(),
            paths: index
                .paths
                .into_iter()
                .flat_map(|(path, methods)| {
                    let path = path.clone();
                    methods
                        .into_iter()
                        .map(move |(method, operation)| PathEntry {
                            path: path.clone(),
                            method,
                            operation,
                        })
                })
                .collect(),
        })
    }

    pub fn write<W: Write>(&self, mut w: W) -> Result {
        let api_version = &self.info.version;
        let server_url = &self.server;
        let prelude = quote! {
            pub const API_VERSION: &str = #api_version;
            pub const SERVER_URL: &str = #server_url;
        };
        writeln!(w, "{}", prelude)?;

        let impls = &self.paths;
        let mut id_items = IdItem::Mod(String::new(), HashMap::new());
        for path in &self.paths {
            let mut cur_item = &mut id_items;
            let parts = path.method_path();
            for part in &parts[0..(parts.len() - 1)] {
                let item = cur_item.expect_mod();
                if !item.contains_key(part) {
                    item.insert(part.clone(), IdItem::Mod(part.clone(), HashMap::new()));
                }
                cur_item = item.get_mut(part).unwrap();
            }
            cur_item
                .expect_mod()
                .insert(path.method_name().clone(), IdItem::Oper(&path));
        }

        let id_items = id_items.expect_mod().into_iter().map(|(_, item)| item);

        let paths = quote! {
            mod inner_impl { #(#impls)* }
            #(#id_items)*
        };
        let config = ts_fmt_lite::ConfigBuilder::default().build().unwrap();
        ts_fmt_lite::print(paths, config, &mut w)?;
        Ok(())
    }
}

struct PathEntry {
    path: String,
    method: String,
    operation: schema::Operation,
}

impl PathEntry {
    fn method_name(&self) -> String {
        use heck::SnakeCase;

        self.operation.operation_id.to_snake_case()
    }

    fn simple_name(&self) -> String {
        use heck::SnakeCase;

        self.operation
            .operation_id
            .split("/")
            .last()
            .unwrap()
            .to_snake_case()
    }

    fn method_path(&self) -> Vec<String> {
        use heck::SnakeCase;

        let mut parts: Vec<String> = self
            .operation
            .operation_id
            .split("/")
            .map(|part| part.to_snake_case())
            .collect();
        *parts.last_mut().unwrap() = self.method_name();
        parts
    }
}

impl ToTokens for PathEntry {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.method_name();
        let name = Ident::new(&name, Span::call_site());
        let summary = &self.operation.summary;
        let description = &self.operation.description;

        let method = &self.method;
        let url = &self.path;

        let q = quote! {
            #[doc = #summary]
            #[doc = ""]
            #[doc = #description]
            pub async fn #name<C: crate::AbstractClient>(client: &C) {
                client._internal_direct(#method, #url).await;
            }
        };
        q.to_tokens(tokens)
    }
}

enum IdItem<'a> {
    Mod(String, HashMap<String, IdItem<'a>>),
    Oper(&'a PathEntry),
}

impl<'a> IdItem<'a> {
    fn expect_mod(&mut self) -> &mut HashMap<String, IdItem<'a>> {
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
        quote! {
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

                    /// API client extension trait for
                    #[doc = #name]
                    /// -related methods
                    ///
                    /// <em>Required feature:
                    #[doc = #name]
                    /// </em>
                    #[cfg(feature = #name)]
                    #[async_trait::async_trait]
                    pub trait #trait_name : crate::AbstractClient {
                        #(#item_trait_fns)*
                    }
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
