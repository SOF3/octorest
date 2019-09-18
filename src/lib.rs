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

use async_trait::async_trait;
use derive_more::From;
use getset::{Getters, Setters};

pub use octorest_routes as routes;

#[derive(Getters, Setters)]
#[get = "pub"]
pub struct Client {
    client: reqwest::Client,
    root_url: String,
}

impl Client {
    pub fn new() -> Self {
        Self::new_with_url(routes::SERVER_URL.into())
    }

    pub fn new_with_url(root_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            root_url,
        }
    }

    pub fn new_with_reqwest(client: reqwest::Client) -> Self {
        Self {
            client,
            root_url: routes::SERVER_URL.into(),
        }
    }

    pub fn new_with_reqwest_url(client: reqwest::Client, root_url: String) -> Self {
        Self { client, root_url }
    }
}

#[async_trait]
impl routes::AbstractClient for Client {
    type Response = ResponseWrapper;

    async fn impl_send(&self, _method: &str, _url: &str) -> ResponseWrapper {
        unimplemented!()
    }

    async fn impl_send_with_body<R>(&self, _method: &str, _url: &str, _body: R) -> ResponseWrapper
    where
        R: IntoIterator<Item = u8> + Send,
    {
        unimplemented!()
    }
}

#[derive(From)]
pub struct ResponseWrapper {
    _inner: reqwest::Response,
}

impl routes::AbstractResponse for ResponseWrapper {
    fn status(&self) -> u16 {
        unimplemented!()
    }
    fn headers(&self) -> Box<dyn Iterator<Item = (String, String)>> {
        unimplemented!()
    }
    fn body(&self) -> Vec<u8> {
        unimplemented!()
    }
}
