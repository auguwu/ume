// 🐻‍❄️💐 ume: Easy, self-hostable, and flexible image host made in Rust
// Copyright 2021-2023 Noel Towa <cutie@floofy.dev>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{rand_string, COMMIT_HASH, VERSION};
use axum::{
    http::{HeaderMap, HeaderName, HeaderValue, Request},
    middleware::Next,
    response::IntoResponse,
};
use std::{ops::Deref, sync::Arc};

/// Represents a request ID.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RequestId(Arc<str>);
impl RequestId {
    /// Returns the [`RequestId`] as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Generates a new [`RequestId`].
    pub fn generate() -> RequestId {
        RequestId(Arc::from(rand_string(24).as_str()))
    }
}

impl Deref for RequestId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub async fn request_id<B>(mut req: Request<B>, next: Next<B>) -> impl IntoResponse {
    let request_id = RequestId::generate();
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("x-request-id"),
        HeaderValue::from_str(&request_id).unwrap(),
    );

    headers.insert(
        HeaderName::from_static("server"),
        HeaderValue::from_str(
            format!("auguwu/ume (+https://github.com/auguwu/ume; v{VERSION}+{COMMIT_HASH})")
                .as_str(),
        )
        .unwrap(),
    );

    req.extensions_mut().insert(request_id);
    (headers, next.run(req).await)
}
