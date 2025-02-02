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
    extract::{FromRequest, Request},
    http::header,
    response::IntoResponse,
    Json, RequestExt,
};
use serde_json::{json, Value};
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

/// Explicit wrapper type for [`multer::Multipart`] that is also an Axum extractor.
pub struct Multipart(multer::Multipart<'static>);
impl Deref for Multipart {
    type Target = multer::Multipart<'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Multipart {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<S: Send + Sync> FromRequest<S> for Multipart {
    type Rejection = Rejection;

    async fn from_request(req: Request, _: &S) -> Result<Self, Self::Rejection> {
        let Some(ct) = req.headers().get(header::CONTENT_TYPE) else {
            return Err(Rejection::NoContentTypeAvaliable);
        };

        let value = ct.to_str().map_err(|e| {
            tracing::error!(error = %e, "received invalid utf-8 in multipart body");
            Rejection::InvalidUtf8ForBoundary
        })?;

        let boundary = multer::parse_boundary(value).map_err(|e| {
            tracing::error!(error = %e, "received invalid multipart body content");
            match e {
                multer::Error::NoBoundary => Rejection::NoBoundary,
                e => Rejection::Multer(e),
            }
        })?;

        let stream = req.with_limited_body().into_body();
        Ok(Self(multer::Multipart::new(stream.into_data_stream(), boundary)))
    }
}

///////////////////////// ERRORS //////////////////////////////////

#[derive(Debug)]
pub enum Rejection {
    /// Error that occurred from the [`multer::Multipart`] instance.
    Multer(multer::Error),

    /// The boundary given was an invalid UTF-8 encoded piece of data.
    InvalidUtf8ForBoundary,

    /// No `Content-Type` header was given in the request.
    NoContentTypeAvaliable,

    /// No multipart boundary was specified.
    NoBoundary,
}

impl Display for Rejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rejection::Multer(err) => Display::fmt(err, f),
            Rejection::NoContentTypeAvaliable => f.write_str("no `content-type` header was specified"),
            Rejection::NoBoundary => f.write_str("received no multipart boundary"),
            Rejection::InvalidUtf8ForBoundary => f.write_str("received invalid utf-8 in multipart boundary decoding"),
        }
    }
}

impl std::error::Error for Rejection {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Rejection::Multer(err) => Some(err),
            _ => None,
        }
    }
}

impl From<multer::Error> for Rejection {
    fn from(value: multer::Error) -> Self {
        Self::Multer(value)
    }
}

impl IntoResponse for Rejection {
    fn into_response(self) -> axum::response::Response {
        Json(json!({
            "message": format!("multipart: {}", match self {
                Self::Multer(ref err) => err_to_msg(err),
                Self::NoContentTypeAvaliable => "no `content-type` header was specified",
                Self::NoBoundary => "received no multipart boundary",
                Self::InvalidUtf8ForBoundary => "received invalid utf-8 in multipart boundary decoding",
            }),

            "details": match self {
                Self::Multer(ref err) => expand_details_from_err(err),
                _ => None,
            }
        }))
        .into_response()
    }
}

pub fn err_to_msg(err: &multer::Error) -> &'static str {
    match err {
        multer::Error::UnknownField { .. } => "received unknown field",
        multer::Error::IncompleteFieldData { .. } => "received incomplete field data in request",
        multer::Error::ReadHeaderFailed(_) => "was unable to read multipart header",
        multer::Error::NoBoundary => "was missing a multipart boundary",
        multer::Error::NoMultipart => "missing `multipart/form-data` contents",
        multer::Error::IncompleteStream => "received incomplete stream, did it corrupt?",
        multer::Error::DecodeContentType(_) => "was unable to decode `Content-Type` header for field",
        multer::Error::DecodeHeaderName { .. } => "decoding header name failed",
        multer::Error::DecodeHeaderValue { .. } => "decoding header value failed",
        multer::Error::FieldSizeExceeded { .. } => "exceeded field size capacity",
        multer::Error::StreamReadFailed(err) => {
            if let Some(err) = err.downcast_ref::<multer::Error>() {
                return err_to_msg(err);
            }

            "reading stream had failed"
        }

        _ => unreachable!(),
    }
}

pub fn expand_details_from_err(err: &multer::Error) -> Option<Value> {
    match err {
        multer::Error::UnknownField { field_name } => field_name.as_ref().map(|field| json!({ "field": field })),
        multer::Error::IncompleteFieldData { field_name } => field_name.as_ref().map(|field| json!({ "field": field })),
        multer::Error::ReadHeaderFailed(_) => None,
        multer::Error::DecodeContentType(_) => None,
        multer::Error::NoBoundary => None,
        multer::Error::NoMultipart => None,
        multer::Error::IncompleteStream => None,
        multer::Error::DecodeHeaderName { name, .. } => Some(json!({ "header": name })),
        multer::Error::StreamSizeExceeded { limit } => Some(json!({ "limit": limit })),
        multer::Error::FieldSizeExceeded { limit, field_name } => field_name.as_ref().map(|field| {
            json!({
                "field": field,
                "limit": limit,
            })
        }),

        multer::Error::StreamReadFailed(err) => {
            if let Some(err) = err.downcast_ref::<multer::Error>() {
                return expand_details_from_err(err);
            }

            None
        }

        _ => None,
    }
}
