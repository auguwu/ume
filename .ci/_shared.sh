#!/usr/bin/env bash

# vendored from:
# https://github.com/charted-dev/charted/blob/fd8ee07a34bb15e3bee3f3f74d5736c460a83154/src/ci/_shared.sh

# 🐻‍❄️📦 charted-server: Free, open source, and reliable Helm Chart registry made in Rust
# Copyright 2022-2025 Noelware, LLC. <team@noelware.org>
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#    http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

function noelware::isMacOS {
    [[ "${OSTYPE}" = darwin* ]]
}

function noelware::isLinux {
    [[ "${OSTYPE}" = linux* ]]
}

function noelware::isWindows {
    [[ "${OSTYPE}" = "cygwin" ]] || [[ "${OSTYPE}" = "msys" ]]
}

function noelware::arch::x86_64 {
    [[ "$(uname -m)" = "amd64" ]] || [[ "$(uname -m)" == "x86_64" ]]
}

function noelware::arch::aarch64 {
    [[ "$(uname -m)" = "arm64" ]] || [[ "$(uname -m)" == "aarch64" ]]
}

function noelware::isCi {
    [[ "${CI-false}" = true ]] || noelware::isGitHubActions
}

function noelware::isGitHubActions {
    [[ "${GITHUB_ACTIONS-false}" = true ]]
}

function noelware::ci::pushenv {
    local name="$1"
    local value=$2

    if noelware::isGitHubActions; then
        echo "SETENV: $name => $value"
        echo "$name=$value" >> "$GITHUB_ENV"
    fi
}

function noelware::startGroup {
    local label="$1"

    if noelware::isGitHubActions; then
        echo "::group::$label"
    else
        echo ">>> $label"
    fi
}

function noelware::endGroup {
    if noelware::isGitHubActions; then
        echo "::endgroup::"
    fi
}
