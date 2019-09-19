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
pub struct Client<H, S>
where
    H: AsRef<reqwest::Client> + Send + Sync,
    S: AsRef<str> + Send + Sync,
{
    http: H,
    root_url: S,
    token: String,
}

impl<H> Client<H, &'static str>
where
    H: AsRef<reqwest::Client> + Send + Sync,
{
    pub fn new(http: H, token: String) -> Self {
        Self::new_with_url(http, routes::SERVER_URL, token)
    }
}

impl<H, S> Client<H, S>
where
    H: AsRef<reqwest::Client> + Send + Sync,
    S: AsRef<str> + Send + Sync,
{
    pub fn new_with_url(http: H, root_url: S, token: String) -> Self {
        Self {
            http,
            root_url,
            token,
        }
    }
}

#[async_trait]
impl<H, S> routes::AbstractClient for Client<H, S>
where
    H: AsRef<reqwest::Client> + Send + Sync,
    S: AsRef<str> + Send + Sync,
{
    type Response = ResponseWrapper;

    async fn impl_send<I>(&self, _method: &str, _url: &str, _headers: I) -> ResponseWrapper
    where
        I: Iterator<Item = (&'static str, String)> + Send,
    {
        unimplemented!()
    }

    async fn impl_send_with_body<R, I>(
        &self,
        _method: &str,
        _url: &str,
        _body: R,
        _headers: I,
    ) -> ResponseWrapper
    where
        R: IntoIterator<Item = u8> + Send,
        I: Iterator<Item = (&'static str, String)> + Send,
    {
        unimplemented!()
    }

    #[inline]
    fn access_token(&self) -> &str {
        self.token() // calling the setter; not a recursion
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
    fn headers(&self) -> Box<dyn Iterator<Item = (&'static str, String)>> {
        unimplemented!()
    }
    fn body(&self) -> Vec<u8> {
        unimplemented!()
    }
}
