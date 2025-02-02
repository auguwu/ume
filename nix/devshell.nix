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
{pkgs}: let
  common = import ./common.nix;
  rustToolchain = common.mkRustPlatform pkgs.rust-bin;
  rustflags = common.rustflags pkgs.stdenv;
in
  with pkgs;
    mkShell {
      LD_LIBRARY_PATH = lib.makeLibraryPath [openssl];

      nativeBuildInputs =
        [pkg-config]
        ++ (lib.optional stdenv.isLinux [mold lldb gdb])
        ++ (lib.optional stdenv.isDarwin [darwin.apple_sdk.frameworks.CoreFoundation]);

      buildInputs =
        [
          cargo-machete
          cargo-expand
          cargo-deny

          rustToolchain
          openssl
          git
        ]
        ++ (lib.optional stdenv.isLinux [glibc]);

      shellHook = ''
        export RUSTFLAGS="${rustflags} $RUSTFLAGS"
      '';
    }
