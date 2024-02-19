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

FROM --platform=${TARGETPLATFORM} rust:1.76-slim-bullseye AS build

ENV DEBIAN_FRONTEND=noninteractive

RUN apt update && apt install -y libssl-dev pkg-config git ca-certificates protobuf-compiler
WORKDIR /build

COPY . .

ENV CARGO_INCREMENTAL=1
ENV RUSTFLAGS="-Ctarget-cpu=native"

RUN cargo build --locked --release --bin charted

############ FINAL STAGE

FROM debian:bullseye-slim

RUN DEBIAN_FRONTEND=noninteractive apt update && apt install -y bash tini curl libssl-dev pkg-config

COPY --from=build /build/target/release/ume /app/noel/ume/bin/ume
COPY distribution/docker/scripts            /app/noel/ume/scripts
COPY distribution/docker/config             /app/noel/ume/config

EXPOSE 3651
VOLUME /var/lib/noel/ume/data

RUN mkdir -p /var/lib/noelware/charted/data
RUN groupadd -g 1001 noelware && \
    useradd -rm -s /bin/bash -g noelware -u 1001 noelware &&  \
    chown noelware:noelware /app/noel/ume &&   \
    chown noelware:noelware /var/lib/noelware/charted/data && \
    chmod +x /app/noel/ume/scripts/docker-entrypoint.sh

# Create a symlink to `ume`
RUN ln -s /app/noel/ume/bin/charted /usr/bin/ume

USER noelware
ENTRYPOINT ["/app/noel/ume/scripts/docker-entrypoint.sh"]
CMD ["ume", "server"]
