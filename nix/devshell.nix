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
  inherit (pkgs) mkShell lib stdenv;
  inherit (lib) makeLibraryPath optional;

  common = import ./common.nix;
  rustflags = common.rustflags stdenv;
  rpath = makeLibraryPath (with pkgs; [openssl]);

  linuxNativeBuildInputs = with pkgs; [mold lldb];
  nativeBuildInputs = (with pkgs; [pkg-config]) ++ linuxNativeBuildInputs;

  buildInputs = with pkgs;
    [
      cargo-upgrades
      cargo-nextest
      cargo-machete
      cargo-expand
      cargo-deny

      (common.mkRustPlatform rust-bin)
      (wrapHelm kubernetes-helm {
        plugins = [charted-helm-plugin];
      })

      openssl
      bun
      git
    ]
    ++ (optional stdenv.isLinux [glibc]);
in
  mkShell {
    inherit buildInputs nativeBuildInputs;

    LD_LIBRARY_PATH = rpath;

    shellHook = ''
      export RUSTFLAGS="${rustflags} $RUSTFLAGS"
    '';
  }
