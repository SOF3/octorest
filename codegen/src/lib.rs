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

extern crate proc_macro;

use std::fs::File;
use std::path::PathBuf;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use syn::{parse_macro_input, Error, LitStr, Result};

mod schema;
mod writer;

#[proc_macro]
pub fn run(path: TokenStream1) -> TokenStream1 {
    let path = parse_macro_input!(path as LitStr);
    let path = path.value();
    let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join(path);

    match inner(path) {
        Ok(ts) => ts,
        Err(err) => err.to_compile_error(),
    }
    .into()
}

fn inner(path: PathBuf) -> Result<TokenStream> {
    let f = File::open(path).map_err(map_err)?;
    let routes = writer::RouteStore::try_new(f)?;
    routes.format()
}

fn map_err<E: std::error::Error>(err: E) -> Error {
    Error::new(Span::call_site(), err)
}
