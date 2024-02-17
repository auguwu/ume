// 🐻‍❄️💐 ume: Easy, self-hostable, and flexible image host made in Rust
// Copyright 2021-2024 Noel Towa <cutie@floofy.dev>
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

use axum::{
    http::{header::AUTHORIZATION, HeaderValue},
    response::IntoResponse,
    Extension, Json,
};
use axum_extra::{headers::Header, TypedHeader};
use noelware_remi::StorageService;
use serde_json::{json, Value};

use super::extract::Multipart;

pub async fn main() -> Json<Value> {
    Json(json!({
        "hello": "world",
        "build_info": json!({
            "version": crate::version(),
            "commit": crate::COMMIT_HASH,
            "build_date": crate::BUILD_DATE
        })
    }))
}

pub async fn heartbeat() -> &'static str {
    "Ok."
}

pub struct UploaderKey(pub HeaderValue);
impl Header for UploaderKey {
    fn name() -> &'static axum::http::HeaderName {
        &AUTHORIZATION
    }

    fn encode<E: Extend<axum::http::HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(
            HeaderValue::from_bytes(self.0.as_ref()).unwrap(),
        ));
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum_extra::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        values
            .next()
            .map(|h| UploaderKey(h.clone()))
            .ok_or_else(axum_extra::headers::Error::invalid)
    }
}

#[instrument(name = "ume.upload.image", skip_all)]
pub async fn upload_image(
    Extension(storage): Extension<StorageService>,
    Extension(config): Extension<crate::config::Config>,
    TypedHeader(header): TypedHeader<UploaderKey>,
    multipart: Multipart,
) -> Result<Json<Value>, Json<Value>> {
    todo!()
}
