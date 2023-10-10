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

use crate::server::Server;
use axum::{
    extract::{FromRequestParts, State},
    http::{header::USER_AGENT, HeaderMap, Method, Request, Uri, Version},
    middleware::Next,
    response::IntoResponse,
};
use std::{sync::atomic::Ordering, time::Instant};
use tracing::{info, info_span};

#[derive(FromRequestParts)]
pub struct Metadata {
    pub(crate) uri: Uri,
    pub(crate) method: Method,
    pub(crate) version: Version,
    pub(crate) headers: HeaderMap,
}

pub async fn log<B>(
    metadata: Metadata,
    State(server): State<Server>,
    req: Request<B>,
    next: Next<B>,
) -> impl IntoResponse {
    server.requests.fetch_add(1, Ordering::Relaxed);

    let uri = metadata.uri.path();
    if uri.contains("/heartbeat") {
        return next.run(req).await;
    }

    let method = metadata.method.as_str();
    let version = match metadata.version {
        Version::HTTP_09 => "http/0.9",
        Version::HTTP_10 => "http/1.0",
        Version::HTTP_11 => "http/1.1",
        Version::HTTP_2 => "http/2.0",
        Version::HTTP_3 => "http/3.0",
        _ => "http/???",
    };

    let ua = metadata
        .headers
        .get(USER_AGENT)
        .map(|f| String::from_utf8_lossy(f.as_bytes()).to_string());

    let http_span = info_span!(
        "ume.http.request",
        req.ua = ua,
        http.uri = uri,
        http.method = method,
        http.version = version
    );

    let _guard = http_span.enter();
    info!("processing request");

    let start = Instant::now();
    let res = next.run(req).await;
    let now = start.elapsed();
    let status = res.status().as_u16();

    info!(http.latency = ?now, res.status = status, "processed request");
    res
}
