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

#![allow(dead_code)]

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use crate::writer::RouteStore;

mod result;
pub use result::*;
mod schema;
mod writer;

fn main() -> Result {
    println!("cargo:rerun-if-changed=ROUTES_TAG");
    let tag = get_build_tag()?;
    let json = get_routes(&tag)?;

    eprintln!("Parsing routes...");
    let routes = RouteStore::try_new(json)?;

    let routes_rs = env_var("OUT_DIR").join("routes.rs");
    eprintln!("Writing to {:?}", routes_rs);
    let mut f = File::create(routes_rs)?;
    routes.write(&mut f)?;
    Ok(())
}

fn env_var(name: &str) -> PathBuf {
    PathBuf::from(std::env::var(name).unwrap())
}

fn get_build_tag() -> Result<String> {
    let mut f = File::open(env_var("CARGO_MANIFEST_DIR").join("ROUTES_TAG")).map_err(map_err)?;
    let mut string = String::new();
    f.read_to_string(&mut string).map_err(map_err)?;
    Ok(string.trim().to_string())
}

fn download_url(tag: &str) -> String {
    format!(
        "https://github.com/octokit/routes/releases/download/{}/api.github.com.json",
        tag
    )
}

fn get_routes(tag: &str) -> Result<impl Read> {
    let dir = env_var("CARGO_MANIFEST_DIR").join("cache");
    if !dir.is_dir() {
        std::fs::create_dir(&dir)?;
    }

    let file = dir.join(&format!("{}.json", tag));
    if file.is_file() {
        eprintln!("Using cache for octokit/routes@{}", tag);
    } else {
        eprintln!("Cache not found for octokit/routes@{}, downloading...", tag);
        let mut f = File::create(&file)?;
        let url = download_url(tag);
        let mut response = reqwest::get(&url).map_err(map_err)?;
        std::io::copy(&mut response, &mut f)?;
    }

    let f = File::open(file)?;
    Ok(f)
}
