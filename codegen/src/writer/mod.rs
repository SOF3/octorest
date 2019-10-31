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

use proc_macro2::TokenStream;
use quote::quote;
use syn::Result;

use crate::schema;
use crate::writer::id_item::IdItem;
use crate::writer::path_entry::PathEntry;

mod id_item;
mod path_entry;

pub struct RouteStore {
    info: schema::Info,
    server: String,
    external_docs: schema::ExternalDocs,
    paths: Vec<PathEntry>,
}

impl RouteStore {
    pub fn try_new<R: Read>(json: R) -> Result<RouteStore> {
        let index: schema::Index = serde_json::from_reader(json).map_err(crate::map_err)?;

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

    pub fn format(&self) -> Result<TokenStream> {
        let api_version = &self.info.version;
        let server_url = &self.server;
        let ext_doc = &self.external_docs;
        let ext_doc_desc = &ext_doc.description;
        let ext_doc_url = &ext_doc.url;
        let prelude = quote! {
            #[doc = "Documentation of"]
            #[doc = #ext_doc_desc]
            pub const DOC_URL: &str = #ext_doc_url;

            /// Version of Rest API (This does not seem to be actually used anywhere in the GitHub API)
            pub const API_VERSION: &str = #api_version;
            /// The root of the API server to be appended with paths
            pub const SERVER_URL: &str = #server_url;
        };

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

        let ret = quote! {
            #prelude
            mod inner_impl { #(#impls)* }
            #(#id_items)*
        };
        Ok(ret)
    }
}

fn remove_doc_tests(string: &str) -> String {
    let mut out = String::with_capacity(string.len());
    let mut flag = true;
    let mut last_pos = 0;
    while let Some(pos) = string[last_pos..].find("```") {
        let pos = last_pos + pos + 3;
        out += &string[last_pos..pos];
        if flag {
            out += "text";
        }
        last_pos = pos;
        flag = !flag;
    }
    out += &string[last_pos..];
    out
}
