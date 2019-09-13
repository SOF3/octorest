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
use derive_new::new;
use getset::{Getters, Setters};

pub use octorest_routes as routes;

#[derive(new, Getters, Setters)]
#[get = "pub"]
pub struct Client {
    access_token: String,
    #[new(value = "routes::SERVER_URL.into()")]
    root_url: String,
}

#[async_trait]
impl routes::AbstractClient for Client {
    type Response = ResponseWrapper;

    async fn _internal_direct(&self, _method: &str, _url: &str) -> ResponseWrapper {
        unimplemented!()
    }

    async fn _internal_data<R>(&self, _method: &str, _url: &str, _data: R) -> ResponseWrapper
    where
        R: IntoIterator<Item = u8> + Send,
    {
        unimplemented!()
    }
}

#[derive(From)]
pub struct ResponseWrapper {
    _inner: reqwest::r#async::Response,
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
