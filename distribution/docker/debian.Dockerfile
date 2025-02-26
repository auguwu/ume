# 🐻‍❄️💐 ume: Easy, self-hostable, and flexible image host made in Rust
# Copyright 2021-2025 Noel Towa <cutie@floofy.dev>
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

############ BINARY

FROM rustlang/rust:nightly-bookworm-slim AS build

ENV DEBIAN_FRONTEND=noninteractive

RUN apt update && apt install -y libssl-dev pkg-config git ca-certificates mold
WORKDIR /build

# So, I decided that `-Zbuild-std` will work in our cases from v4.1 and higher. For now,
# we will use a Nightly compilter as `-Zbuild-std` is a Nightly feature, which is fine.

# We need `rust-src` to be avaliable.
RUN rustup component add rust-src

ENV RUSTFLAGS="-Clink-arg=-fuse-ld=mold -Ctarget-cpu=native"

# So, we would like to cache `/build/target` so we don't have to build the
# whole project. So, we will need to create a dummy project for now.
COPY Cargo.toml .
RUN mkdir -p src/ && \
    echo "fn main() {}" > src/dummy.rs && \
    sed -i 's#src/bin/ume.rs#src/dummy.rs#' Cargo.toml

RUN --mount=type=cache,target=/build/target/                                 \
    --mount=type=cache,target=/usr/local/cargo/git/db                        \
    --mount=type=cache,target=/usr/local/cargo/registry/                     \
    cargo build                                                              \
    -Zbuild-std=std,panic_abort                                              \
    -Zbuild-std-features="optimize_for_size,panic_immediate_abort,backtrace" \
    --no-default-features                                                    \
    --release

# Now that our dependencies should be cached (hopefully), so let's add back
# the actual binary target
RUN rm src/dummy.rs && sed -i 's#src/dummy.rs#src/bin/ume.rs#' Cargo.toml

COPY . .

# We need the `rust-toolchain.toml` file removed since it'll overwrite it.
RUN rm rust-toolchain.toml

# Now, let's rebuild the `ume` binary.
RUN cargo build                                                              \
    -Zbuild-std=std,panic_abort                                              \
    -Zbuild-std-features="optimize_for_size,panic_immediate_abort,backtrace" \
    --locked                                                                 \
    --no-default-features                                                    \
    --release

############ FINAL STAGE

FROM debian:bookworm-slim

RUN DEBIAN_FRONTEND=noninteractive apt update && apt install -y bash tini curl libssl-dev pkg-config
WORKDIR /app/noel/ume

COPY --from=build /build/target/release/ume /app/noel/ume/bin/ume
COPY distribution/docker/scripts            /app/noel/ume/scripts
COPY distribution/docker/config             /app/noel/ume/config

EXPOSE 3621
VOLUME /var/lib/noel/ume/data
ENV UME_STORAGE_FILESYSTEM_DIRECTORY=/var/lib/noel/ume/data

RUN mkdir -p /var/lib/noel/ume/data
RUN groupadd -g 1001 noel && \
    useradd -rm -s /bin/bash -g noel -u 1001 noel &&  \
    chown -R noel:noel /app/noel/ume && \
    chown -R noel:noel /var/lib/noel/ume/data && \
    chmod +x /app/noel/ume/scripts/docker-entrypoint.sh

# Create a symlink to `ume`
RUN ln -s /app/noel/ume/bin/ume /usr/bin/ume

USER noel
ENTRYPOINT ["/app/noel/ume/scripts/docker-entrypoint.sh"]
CMD ["/app/noel/ume/bin/ume", "server"]
