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

#[async_trait]
impl routes::AbstractClient for Client {
    type Response = ResponseWrapper;

    async fn impl_send<I>(&self, _method: &str, _url: &str, headers: I) -> ResponseWrapper
    where
        I: Iterator<Item = (String, String)>,
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
        I: Iterator<Item = (String, String)>,
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
