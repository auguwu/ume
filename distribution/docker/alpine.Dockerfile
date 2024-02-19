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

############ BINARY

FROM --platform=${TARGETPLATFORM} rust:1.76-alpine3.19 AS build

RUN apk update && apk add --no-cache git ca-certificates curl musl-dev libc6-compat gcompat pkgconfig openssl-dev build-base
WORKDIR /build

COPY . .

# Remove the `rust-toolchain.toml` file since we expect to use `rustc` from the Docker image
# rather from rustup.
RUN rm rust-toolchain.toml

ENV RUSTFLAGS="-Ctarget-cpu=native -Ctarget-feature=-crt-static"
RUN cargo build --release --bin ume

############ FINAL STAGE

FROM alpine:3.19

RUN apk update && apk add --no-cache bash tini curl libgcc
WORKDIR /app/noel/ume

COPY --from=build /build/target/release/charted /app/noel/ume/bin/ume
COPY distribution/docker/scripts                /app/noel/ume/scripts
COPY distribution/docker/config                 /app/noel/ume/config

EXPOSE 3621
VOLUME /var/lib/noel/ume/data
ENV UME_STORAGE_FILESYSTEM_DIRECTORY=/var/lib/noel/ume/data

RUN mkdir -p /var/lib/noel/ume/data
RUN addgroup -g 1001 noel && \
    adduser -DSH -u 1001 -G noel noel && \
    chown -R noel:noel /app/noel/ume && \
    chown -R noel:noel /var/lib/noel/ume/data && \
    chmod +x /app/noel/ume/scripts/docker-entrypoint.sh

# Create a symbolic link so you can just run `ume` without specifying
# the full path.
RUN ln -s /app/noel/ume/bin/ume /usr/bin/ume

USER noel
ENTRYPOINT ["/app/noel/ume/scripts/docker-entrypoint.sh"]
CMD ["/app/noel/ume/bin/ume", "server"]
