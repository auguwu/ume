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
{
  cargoTOML = builtins.fromTOML (builtins.readFile ../Cargo.toml);
  outputHashes = {
    "azalia-0.1.0" = "sha256-+fvpmHxO/QPQlDU7GMARx3oIJuEcCjrDj4xfaUc4CX8=";
  };

  rustflags = stdenv:
    if stdenv.isLinux
    then ''-C link-arg=-fuse-ld=mold -C target-cpu=native $RUSTFLAGS''
    else ''$RUSTFLAGS'';

  mkRustPlatform = rust: rust.fromRustupToolchainFile ../rust-toolchain.toml;
  mkNixpkgsRustPlatform = pkgs: toolchain:
    pkgs.makeRustPlatform {
      rustc = toolchain;
      cargo = toolchain;
    };
}
