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

use crate::config::{self, tracing::otel::Kind, Config};
use axum::{
    body::Body,
    extract::DefaultBodyLimit,
    http::{header, Response, StatusCode},
    Extension,
};
use axum_server::{tls_rustls::RustlsConfig, Handle};
use eyre::Context;
use noelware_log::{writers, WriteLayer};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::TracerProvider;
use owo_colors::{OwoColorize, Stream::Stdout};
use remi::StorageService;
use sentry::types::Dsn;
use serde_json::json;
use std::{
    any::Any,
    borrow::Cow,
    io::{self, Write as _},
    path::PathBuf,
    str::FromStr,
    time::Duration,
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

/// Starts a Ume server
#[derive(Debug, Clone, clap::Parser)]
pub struct Cmd {
    /// location to a `config.hcl` file
    #[arg(long, short = 'c', env = "UME_CONFIG_FILE")]
    config: Option<PathBuf>,

    /// list of Tokio workers to use, this will be limited to your CPU cores.
    #[arg(long, short = 'w', env = "UME_SERVER_WORKERS")]
    pub workers: Option<usize>,
}

pub async fn execute(cmd: Cmd) -> eyre::Result<()> {
    let loc = match cmd.config {
        Some(ref path) => format!("in path [{}]", path.display()),
        None => match Config::find_default_location() {
            Some(path) => format!("in path [{}]", path.display()),
            None => String::from("via system environment variables"),
        },
    };

    let config = match cmd.config {
        Some(ref path) => Config::new(Some(path)),
        None => match Config::find_default_location() {
            Some(path) => Config::new(Some(path)),
            None => Config::new::<&str>(None),
        },
    }?;

    print_banner();
    let _sentry_guard = sentry::init(sentry::ClientOptions {
        traces_sample_rate: match config.tracing {
            config::tracing::Config::Sentry(config::tracing::sentry::Config { sample_set }) => {
                sample_set
            }
            _ => 0.5,
        },
        attach_stacktrace: true,
        server_name: Some(Cow::Borrowed("ume")),
        release: Some(Cow::Borrowed(crate::version())),
        dsn: config
            .sentry_dsn
            .as_ref()
            .map(|dsn| Dsn::from_str(dsn).expect("valid Sentry DSN")),

        ..Default::default()
    });

    let provider = if let config::tracing::Config::OpenTelemetry(ref otel) = config.tracing {
        let mut provider = TracerProvider::builder();
        match otel.kind {
            Kind::Grpc => {
                provider = provider.with_simple_exporter(
                    opentelemetry_otlp::new_exporter()
                        .tonic()
                        .build_span_exporter()?,
                )
            }

            Kind::Http => {
                provider = provider.with_simple_exporter(
                    opentelemetry_otlp::new_exporter()
                        .http()
                        .build_span_exporter()?,
                )
            }
        };

        Some(provider.build())
    } else {
        None
    };

    tracing_subscriber::registry()
        .with(
            match config.logging.json {
                false => WriteLayer::new_with(io::stdout(), writers::default),
                true => WriteLayer::new_with(io::stdout(), writers::json),
            }
            .with_filter(LevelFilter::from_level(config.logging.level))
            .with_filter(tracing_subscriber::filter::filter_fn(|meta| {
                // disallow from getting logs from `tokio` since it doesn't contain anything
                // useful to us
                !meta.target().starts_with("tokio::")
            })),
        )
        .with(sentry_tracing::layer())
        .with(
            config
                .logging
                .logstash_tcp_uri
                .as_ref()
                .map(|url| {
                    let stream = std::net::TcpStream::connect(url).unwrap();
                    WriteLayer::new_with(stream, writers::json)
                })
                .with_filter(LevelFilter::from_level(config.logging.level))
                .with_filter(tracing_subscriber::filter::filter_fn(|meta| {
                    // disallow from getting logs from `tokio` since it doesn't contain anything
                    // useful to us
                    !meta.target().starts_with("tokio::")
                })),
        )
        .with(
            provider
                .map(|provider| tracing_opentelemetry::layer().with_tracer(provider.tracer("ume"))),
        )
        .init();

    info!("loaded configuration from {loc}, starting Ume server...");
    let storage = match config.storage {
        crate::config::storage::Config::Filesystem(ref fs) => {
            noelware_remi::StorageService::Filesystem(remi_fs::StorageService::with_config(
                fs.clone(),
            ))
        }

        crate::config::storage::Config::Azure(ref azure) => {
            noelware_remi::StorageService::Azure(remi_azure::StorageService::new(azure.clone()))
        }

        crate::config::storage::Config::GridFS(ref gridfs) => {
            let client = mongodb::Client::with_options(gridfs.client_options.clone())?;

            noelware_remi::StorageService::GridFS(remi_gridfs::StorageService::from_client(
                &client,
                gridfs.clone(),
            ))
        }

        crate::config::storage::Config::S3(ref s3) => {
            noelware_remi::StorageService::S3(remi_s3::StorageService::new(s3.clone()))
        }
    };

    storage.init().await?;

    let router = crate::server::create_router()
        .layer(sentry_tower::NewSentryLayer::new_from_top())
        .layer(sentry_tower::SentryHttpLayer::with_transaction())
        .layer(tower_http::catch_panic::CatchPanicLayer::custom(
            panic_handler,
        ))
        .layer(DefaultBodyLimit::max(15 * 1024 * 1024))
        .layer(axum::middleware::from_fn(crate::server::middleware::log))
        .layer(axum::middleware::from_fn(
            crate::server::middleware::request_id,
        ))
        .layer(Extension(storage))
        .layer(Extension(config.clone()));

    if let Some(ref cfg) = config.server.ssl {
        info!("server is now using HTTPS support");

        // keep a handle for the TLS server so the shutdown signal can all shutdown
        let handle = axum_server::Handle::new();
        tokio::spawn(shutdown_signal(Some(handle.clone())));

        let addr = config.server.addr();
        let config = RustlsConfig::from_pem_file(&cfg.cert, &cfg.cert_key).await?;

        info!(address = ?addr, "listening on HTTPS");
        axum_server::bind_rustls(addr, config)
            .handle(handle)
            .serve(router.into_make_service())
            .await
    } else {
        let addr = config.server.addr();
        let listener = tokio::net::TcpListener::bind(addr).await?;
        info!(address = ?addr, "listening on HTTP");

        axum::serve(listener, router.into_make_service())
            .with_graceful_shutdown(shutdown_signal(None))
            .await
    }
    .context("unable to run HTTP service")
}

async fn shutdown_signal(handle: Option<Handle>) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("unable to install CTRL+C handler");
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
    let details = if let Some(msg) = message.downcast_ref::<String>() {
        msg.clone()
    } else if let Some(msg) = message.downcast_ref::<&str>() {
        msg.to_string()
    } else {
        "unable to downcast message".to_string()
    };

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

fn print_banner() {
    let mut stdout = io::stdout().lock();
    let _ = writeln!(
        stdout,
        "» Booting up {} v{}, compiled with Rust v{}",
        "ume".if_supports_color(Stdout, |x| x.bold()),
        crate::version().if_supports_color(Stdout, |x| x.bold()),
        crate::RUSTC.if_supports_color(Stdout, |x| x.bold())
    );
}
