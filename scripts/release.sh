#!/usr/bin/env bash

# 🐻‍❄️💐 ume: Easy, self-hostable, and flexible image host made in Rust
# Copyright 2021-2024 Noel Towa <cutie@floofy.dev>
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# This is the release pipeline for building the `ume` binary
# on GitHub Actions instead of relying on the YAML representation.

target=${BUILDTARGET:-"<unknown>"}
flags=${BUILDFLAGS:-}
cargo=${CARGO:-cargo}
os=""
arch=""

case "$(uname -s)" in
    Linux)
        os="linux";
        ;;
    Darwin)
        os="darwin";
        ;;
    *)
        echo "===> ERROR: unsupported OS: \`$(uname -s)\`"
        exit 1
        ;;
esac

case "$(uname -m)" in
    x86_64|amd64)
        arch="x86_64";
        ;;
    aarch64|arm64)
        arch="aarch64";
        ;;
    *)
        echo "===> ERROR: unsupported architecture: \`$(uname -m)\`"
        exit 1
        ;;
esac

if ! command -v "$cargo" >/dev/null; then
    echo "===> ERROR: Missing \`cargo\` binary (defined as \`\$CARGO\`: $cargo)"
    exit 1
fi

function ume::build {
    [ "$target" == "<unknown>" ] && {
        echo "===> ERROR: \`./scripts/release.sh\` requires a target to be set via \`\$BUILDTARGET\` environment variable."
        exit 1
    }

    # Update the `$arch` variable to `aarch64` on macOS since it'll detect as we are using
    # the Intel chip of macOS since the M1 runners require a GitHub Teams or Enterprise license,
    # so we'll hack our way there.
    if [[ "$os" == "darwin" && "$target" == "aarch64-apple-darwin" && "$arch" == "x86_64" ]]; then
        arch="aarch64"
    fi

    # ...also update `$arch` for the arm64 build of Ume on Linux since cross-rs uses the host system
    # with the right compilers, so we need to update it.
    if [[ "$os" == "linux" && "$cargo" == "cross" ]]; then
        arch="aarch64"
    fi

    ! [ -d "./.result" ] && mkdir -p ./.result
    pushd ./.result >/dev/null

    echo "===> Compiling release \`ume\` binary                 [target=$target] [flags=$flags] [\$CARGO=$cargo] [os=$os] [arch=$arch]"
    echo "$ $cargo build --release --locked --target $target $flags"
    "$cargo" build --release --locked --target="$target" $flags || exit 1

    echo "Moving ./target/$target/release/ume ~> .result/ume-$os-$arch"
    mv ../target/"$target"/release/ume ./"ume-$os-$arch" || exit 1

    echo "===> Generating sha256sum file                          [binary=ume-$os-$arch]"
    if [ "$(uname -s)" == "Darwin" ]; then
        shasum -a 256 "ume-$os-$arch" > ./"ume-$os-$arch.sha256"
    else
        sha256sum "ume-$os-$arch" > ./"ume-$os-$arch.sha256"
    fi

    echo "===> Created SHA256 file for binary                     [binary=ume-$os-$arch]"
    echo "===> Completed."

    popd >/dev/null
}

ume::build
