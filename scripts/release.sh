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
        arch="x86_64"
        ;;
    aarch64|arm64)
        arch="arm64";
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

    ! [ -d "./.result" ] && mkdir -p ./.result
    pushd ./.result >/dev/null

    # Export $RUSTFLAGS so we can use the target's CPU instructions
    export RUSTFLAGS=""
    extra=""
    if [ "$(uname -s)" == "Linux" ]; then
        # ...and use `mold` as the linker since it is faster
        export RUSTFLAGS="-Clink-arg=-fuse-ld=mold $RUSTFLAGS"

        if [ "$target" == "x86_64-unknown-linux-musl" ]; then
            extra="-musl"
        else
            extra="-gnu"
        fi
    fi

    echo "===> Compiling release \`ume\` binary                 [target=$target] [flags=$flags] [\$CARGO=$cargo]"
    echo "$ $cargo build --release --locked --target $target $flags"
    "$cargo" build --release --locked --target="$target" $flags || exit 1

    echo "Moving ./target/$target/release/ume ~> .result/ume-$os-$arch$extra"
    mv ../target/"$target"/release/ume ./"ume-$os-$arch$extra" || exit 1

    shacmd="sha256sum"
    if [ "$(uname -s)" == "Darwin" ]; then
        # macOS is weird ok
        shacmd="shasum -256"
    fi

    echo "$ $shacmd ume-$os-$arch$extra"
    "$shacmd" "ume-$os-$arch$extra" > ./"ume-$os-$arch$extra.sha256"

    echo "===> Created SHA256 file for binary                     [binary=$ume-$os-$arch$extra]"
    echo "===> Completed."

    popd >/dev/null
}

ume::build
