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
{
  pkg-config,
  installShellFiles,
  openssl,
  lib,
  makeRustPlatform,
  rust-bin,
}: let
  common = import ./common.nix;
  rustToolchain = common.mkRustPlatform rust-bin;
  rustPlatform = common.mkNixpkgsRustPlatform {inherit makeRustPlatform;} rustToolchain;
  version = common.cargoTOML.package.version;
in
  rustPlatform.buildRustPackage {
    inherit version;

    pname = "ume";
    src = ../.;

    nativeBuildInputs = [pkg-config installShellFiles];
    buildInputs = [openssl];

    cargoLock = {
      inherit (common) outputHashes;

      lockFile = ../Cargo.lock;
    };

    postInstall = ''
      installShellCompletion --cmd ume \
        --bash <($out/bin/ume completions bash) \
        --fish <($out/bin/ume completions fish) \
        --zsh <($out/bin/ume completions zsh)
    '';

    meta = with lib; {
      description = "Easy, self-hostable, and flexible image host made in Rust";
      homepage = "https://github.com/auguwu/ume";
      license = with licenses; [asl20];
      maintainers = with maintainers; [auguwu];
      mainProgram = "ume";
    };
  }
