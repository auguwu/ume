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
use azalia::log::{writers, WriteLayer};
use opentelemetry::{trace::TracerProvider as _, KeyValue};
use opentelemetry_sdk::trace::TracerProvider;
use owo_colors::{OwoColorize, Stream::Stdout};
use remi::StorageService;
use sentry::types::Dsn;
use std::{
    borrow::Cow,
    io::{self, Write as _},
    path::PathBuf,
    str::FromStr,
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
            config::tracing::Config::Sentry(config::tracing::sentry::Config { sample_set }) => sample_set,
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

    let tracer = if let config::tracing::Config::OpenTelemetry(ref otel) = config.tracing {
        let mut provider = TracerProvider::builder();
        match otel.kind {
            Kind::Grpc => {
                provider =
                    provider.with_simple_exporter(opentelemetry_otlp::new_exporter().tonic().build_span_exporter()?)
            }

            Kind::Http => {
                provider =
                    provider.with_simple_exporter(opentelemetry_otlp::new_exporter().http().build_span_exporter()?)
            }
        };

        let provider = provider.build();
        let mut attributes = otel
            .labels
            .iter()
            .map(|(key, value)| KeyValue::new(key.clone(), value.clone()))
            .collect::<Vec<_>>();

        attributes.push(KeyValue::new("service.name", "ume"));
        attributes.push(KeyValue::new("ume.version", crate::version()));

        Some(provider.versioned_tracer(
            "ume",
            Some(crate::version()),
            Some("https://opentelemetry.io/schema/1.0.0"),
            Some(attributes),
        ))
    } else {
        None
    };

    tracing_subscriber::registry()
        .with(
            match config.logging.json {
                false => WriteLayer::new_with(io::stdout(), writers::default::Writer::default()),
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
        .with(tracer.map(|tracer| {
            tracing_opentelemetry::layer()
                .with_tracer(tracer)
                .with_filter(LevelFilter::from_level(config.logging.level))
        }))
        .init();

    info!("loaded configuration from {loc}, starting Ume server...");
    let storage = match config.storage.clone() {
        crate::config::storage::Config::Filesystem(fs) => {
            azalia::remi::StorageService::Filesystem(remi_fs::StorageService::with_config(fs))
        }

        crate::config::storage::Config::Azure(azure) => {
            azalia::remi::StorageService::Azure(remi_azure::StorageService::new(azure))
        }

        crate::config::storage::Config::GridFS(gridfs) => {
            let client = mongodb::Client::with_options(gridfs.client_options.clone())?;
            azalia::remi::StorageService::GridFS(remi_gridfs::StorageService::from_client(&client, gridfs))
        }

        crate::config::storage::Config::S3(s3) => azalia::remi::StorageService::S3(remi_s3::StorageService::new(s3)),
    };

    storage.init().await?;
    crate::server::start_server(storage, config).await
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
