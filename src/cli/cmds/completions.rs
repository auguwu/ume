// 🐻‍❄️💐 ume: Easy, self-hostable, and flexible image host made in Rust
// Copyright 2021-2025 Noel Towa <cutie@floofy.dev>
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

use crate::cli::Program;
use azalia::config::env::{self, TryParseError};
use clap::CommandFactory;
use clap_complete::Shell;
use std::{io, path::PathBuf};

/// Generates shell completions for any shell. This doesn't support nushell or fig.
#[derive(Debug, Clone, clap::Parser)]
pub struct Cmd {
    /// A shell to generate for, this will default to `$SHELL` that is defined.
    shell: Option<Shell>,
}

pub fn execute(cmd: Cmd) -> eyre::Result<()> {
    let default_shell = match env::try_parse_or_else::<_, PathBuf>("SHELL", "/usr/bin/sh".into()) {
        Ok(path) => Shell::from_shell_path(path).unwrap_or(Shell::Bash),
        Err(TryParseError::System(_)) => bail!("received invalid unicode for `$SHELL`"),
        Err(TryParseError::Parse(_)) => unreachable!(), // it should never fail
    };

    let shell = cmd.shell.unwrap_or(default_shell);
    trace!(%shell, "generating shell completions for");

    {
        let mut cmd = Program::command();
        let mut stdout = io::stdout().lock();
        clap_complete::generate(shell, &mut cmd, "ume", &mut stdout);
    }

    Ok(())
}
