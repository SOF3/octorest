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

    async fn impl_send(&self, method: &str, url: &str) -> Self::Response;

    async fn impl_send_with_body<R>(&self, method: &str, url: &str, data: R) -> Self::Response
    where
        R: IntoIterator<Item = u8> + Send;
}

pub trait AbstractResponse: Sized + Send {
    fn status(&self) -> u16;

    fn headers(&self) -> Box<dyn Iterator<Item = (String, String)>>;

    fn body(&self) -> Vec<u8>;
}