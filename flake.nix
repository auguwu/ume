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
  description = "🐻‍❄️💐 Easy, self-hostable, and flexible image host made in Rust";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    noelware = {
      url = "github:Noelware/nixpkgs-noelware";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  # Include the binary caches for my own projects and Noelware's as well.
  nixConfig = {
    extra-substituters = [
      # TODO: switch to https://nix.noel.pink
      "https://noel.cachix.org"

      # TODO: switch to https://nix.noelware.org
      "https://noelware.cachix.org"
    ];

    extra-trusted-public-keys = [
      "noel.cachix.org-1:pQHbMJOB5h5VqYi3RV0Vv0EaeHfxARxgOhE9j013XwQ="
      "noelware.cachix.org-1:22A8ELRjkqEycSHz+R5A5ReX2jyjU3rftsBmlD6thn0="
    ];
  };

  outputs = {
    nixpkgs,
    systems,
    rust-overlay,
    noelware,
    ...
  }: let
    eachSystem = nixpkgs.lib.genAttrs (import systems);
    overlays = [
      (import rust-overlay)
      (import noelware)
    ];

    nixpkgsFor = system:
      import nixpkgs {
        inherit system overlays;
      };
  in {
    formatter = eachSystem (system: (nixpkgsFor system).alejandra);
    packages = eachSystem (system: let
      pkgs = nixpkgsFor system;
      ume = pkgs.callPackage ./nix/package.nix {};
    in {
      inherit ume;

      default = ume;
    });

    devShells = eachSystem (system: let
      pkgs = nixpkgsFor system;
    in {
      default = import ./nix/devshell.nix {inherit pkgs;};
    });
  };
}
