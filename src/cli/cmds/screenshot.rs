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

use chrono::Local;
use eyre::Context;
use indicatif::ProgressStyle;
use std::{
    fs::{self, create_dir_all, remove_file, OpenOptions},
    path::PathBuf,
    process::{exit, Command, Stdio},
};
use tracing_indicatif::span_ext::IndicatifSpanExt;
use url::Url;

/// Takes a screenshot with [Flameshot](https://flameshot.org)
#[derive(Debug, Clone, clap::Parser)]
pub struct Cmd {
    /// URL that points to a Ume server
    #[arg(env = "UME_SERVER")]
    server: Url,

    /// uploader key to upload images to a Ume server
    #[arg(long = "master-key", env = "UME_UPLOADER_KEY")]
    master_key: String,

    /// directory to place all screenshots in, this will default to `$TMPDIR`.
    #[arg(long, short = 'd', env = "UME_TEMP_DIRECTORY")]
    tempdir: Option<PathBuf>,

    /// path to a `flameshot` executable, defaults to one that is found in `$PATH`
    #[arg(long, env = "FLAMESHOT")]
    flameshot: Option<PathBuf>,

    /// disables copying the image URL or the image itself (if it failed to upload)
    #[arg(long, env = "UME_NO_COPY")]
    no_copy: bool,
}

pub async fn execute(cmd: Cmd) -> eyre::Result<()> {
    let tempdir = cmd.tempdir.unwrap_or(std::env::temp_dir());
    let screenshots = tempdir.join("screenshots");
    if !screenshots.try_exists()? {
        create_dir_all(&screenshots)?;
    }

    let flameshot = cmd
        .flameshot
        .unwrap_or(which::which("flameshot").context("unable to find `flameshot` program")?);

    info!(flameshot = %flameshot.display(), "found `flameshot` program :3");
    let name = screenshots.join(format!("{}.png", Local::now().to_rfc3339()));
    info!(file = %name.display(), "creating file...");

    let file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(screenshots.join(&name))?;

    info!("$ {} gui -r > {}", flameshot.display(), name.display());

    let mut cmd = Command::new(&flameshot);
    cmd.args(["gui", "-r"])
        .stdout(file.try_clone()?)
        .stderr(Stdio::inherit())
        .stdin(Stdio::null());

    let output = cmd.output()?;
    if !output.status.success() {
        error!(file = %name.display(), "failed to run `flameshot gui -r` onto file");
        remove_file(&name)?;

        exit(1);
    }

    info!(file = %name.display(), "uploading file to Ume server...");

    let len = file.metadata()?.len();

    // upload the image
    let progress = info_span!("ume.screenshot.upload", file = %name.display());
    progress.pb_set_style(
        &ProgressStyle::with_template("{wide_bar:.cyan/blue} {bytes}/{total_bytes} {elapsed}")
            .expect("to compile progress style template")
            .progress_chars("#>-")
            .with_key("elapsed", crate::cli::elapsed_subsec),
    );

    progress.pb_set_length(len);

    // it no longer should be available since it'll be copied to
    // the clipboard if the `cfg(feature = "clipboard")` feature
    // is enabled or if `--no-copy` is specified.
    fs::remove_file(name).context("unable to delete file")
}
