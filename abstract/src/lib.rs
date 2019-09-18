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

use std::iter::IntoIterator;

#[async_trait::async_trait]
pub trait AbstractClient: Sized + Send + Sync {
    type Response: AbstractResponse;

    async fn impl_send<I>(&self, method: &str, url: &str, headers: I) -> Self::Response
    where
        I: Iterator<Item = (&'static str, String)>;

    async fn impl_send_with_body<R, I>(
        &self,
        method: &str,
        url: &str,
        data: R,
        headers: I,
    ) -> Self::Response
    where
        R: IntoIterator<Item = u8> + Send,
        I: Iterator<Item = (&'static str, String)>;

    fn access_token(&self) -> &str;
}

pub trait AbstractResponse: Sized + Send {
    fn status(&self) -> u16;

    fn headers(&self) -> Box<dyn Iterator<Item = (&'static str, String)>>;

    fn body(&self) -> Vec<u8>;
}

pub mod internals {
    use std::collections::HashMap;
    use std::iter::IntoIterator;

    use crate::AbstractClient;

    pub fn normal_headers<'a, C>(client: &'a C, accept: &'static str) -> Headers<'a>
    where
        C: AbstractClient,
    {
        Headers {
            state: HeadersIterState::default(),
            token: client.access_token(),
            accept,
            extra: Some(HashMap::new()),
        }
    }

    #[derive(Debug)]
    pub struct Headers<'a> {
        state: HeadersIterState,
        token: &'a str,
        accept: &'static str,
        extra: Option<HashMap<&'static str, String>>,
    }

    impl<'a> Iterator for Headers<'a> {
        type Item = (&'static str, String);

        fn next(&mut self) -> Option<(&'static str, String)> {
            match &mut self.state {
                HeadersIterState::AccessToken => {
                    self.state = HeadersIterState::ContentType;
                    Some(("Authorization", format!("bearer {}", self.token)))
                }
                HeadersIterState::ContentType => {
                    self.state = HeadersIterState::Extra(self.extra.take().unwrap().into_iter());
                    Some(("Accept", self.accept.into()))
                }
                HeadersIterState::Extra(iter) => iter.next(),
            }
        }
    }

    #[derive(Debug)]
    enum HeadersIterState {
        AccessToken,
        ContentType,
        Extra(<HashMap<&'static str, String> as IntoIterator>::IntoIter),
    }

    impl Default for HeadersIterState {
        fn default() -> Self {
            Self::AccessToken
        }
    }
}
