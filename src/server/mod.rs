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

mod config;
pub use config::*;

mod extract;
mod middleware;
mod routes;

use axum::{
    body::Body,
    extract::DefaultBodyLimit,
    http::{header, Response, StatusCode},
    routing, Extension, Router,
};
use axum_server::{tls_rustls::RustlsConfig, Handle};
use azalia::remi::StorageService;
use eyre::Context;
use serde_json::json;
use std::{any::Any, time::Duration};

pub fn create_router() -> Router {
    Router::new()
        .route("/heartbeat", routing::get(routes::heartbeat))
        .route("/images/upload", routing::post(routes::upload_image))
        .route("/images/{name}", routing::get(routes::get_image))
        .route("/", routing::get(routes::main))
}

/// Starts a Ume server with the configured [`StorageService`] and loaded configuration file.
pub async fn start_server(storage: StorageService, config: crate::config::Config) -> eyre::Result<()> {
    info!("starting Ume server!");

    let router = create_router()
        .layer(sentry_tower::NewSentryLayer::new_from_top())
        .layer(sentry_tower::SentryHttpLayer::with_transaction())
        .layer(tower_http::catch_panic::CatchPanicLayer::custom(panic_handler))
        .layer(DefaultBodyLimit::max(15 * 1024 * 1024))
        .layer(axum::middleware::from_fn(crate::server::middleware::log))
        .layer(axum::middleware::from_fn(crate::server::middleware::request_id))
        .layer(Extension(storage))
        .layer(Extension(config.clone()));

    match config.server.ssl {
        Some(ref ssl) => start_https_server(&config.server, ssl, router.clone()).await,
        None => start_http_server(&config.server, router).await,
    }
}

async fn start_https_server(config: &Config, ssl: &config::ssl::Config, router: Router) -> eyre::Result<()> {
    let handle = Handle::new();
    tokio::spawn(shutdown_signal(Some(handle.clone())));

    let addr = config.addr();
    let config = RustlsConfig::from_pem_file(&ssl.cert, &ssl.cert_key).await?;

    info!(address = %addr, "listening on HTTPS");
    axum_server::bind_rustls(addr, config)
        .handle(handle)
        .serve(router.into_make_service())
        .await
        .context("failed to run HTTPS server")
}

async fn start_http_server(config: &Config, router: Router) -> eyre::Result<()> {
    let addr = config.addr();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(address = ?addr, "listening on HTTP");

    axum::serve(listener, router.into_make_service())
        .with_graceful_shutdown(shutdown_signal(None))
        .await
        .context("failed to run HTTP server")
}

async fn shutdown_signal(handle: Option<Handle>) {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("unable to install CTRL+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("unable to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }

    warn!("received terminal signal! shutting down");
    if let Some(handle) = handle {
        handle.graceful_shutdown(Some(Duration::from_secs(10)));
    }
}

fn panic_handler(message: Box<dyn Any + Send + 'static>) -> Response<Body> {
    let details = azalia::message_from_panic(message);

    error!(%details, "route had panic'd");
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
        .body(Body::from(
            serde_json::to_string(&json!({
                "message": "ume server had failed to do your request, please try again later"
            }))
            .unwrap(),
        ))
        .unwrap()
}
