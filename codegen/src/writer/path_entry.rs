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

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

use crate::schema;

pub struct PathEntry {
    pub path: String,
    pub method: String,
    pub operation: schema::Operation,
}

impl PathEntry {
    pub fn method_name(&self) -> String {
        use heck::SnakeCase;

        self.operation.operation_id.to_snake_case()
    }

    pub fn simple_name(&self) -> String {
        use heck::SnakeCase;

        self.operation
            .operation_id
            .split("/")
            .last()
            .unwrap()
            .to_snake_case()
    }

    pub fn method_path(&self) -> Vec<String> {
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
        let description = self
            .operation
            .description
            .as_ref()
            .map(|string| remove_doc_tests(string));

        let method = &self.method;
        let url = &self.path;
        let accept = "application/vnd.github.v3+json"; // TODO parse args

        let q = quote! {
            #[doc = #summary]
            #[doc = ""]
            #[doc = #description]
            pub async fn #name<C: crate::AbstractClient>(client: &C) {
                let headers = crate::normal_headers(client, #accept);
                client.impl_send(#method, #url, headers).await;
            }
        };
        q.to_tokens(tokens)
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
