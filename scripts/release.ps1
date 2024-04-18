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

# This script is the same as `release.sh` but for Windows.

# We don't need a `-Target ...` parameter as we only build for x86_64 Windows only.
function Main {
    if (![System.Environment]::Is64BitOperatingSystem) {
        Write-Error "FATAL: 'ume' is not supported on x86 systems!"
        Exit 1
    }

    $Cargo = [System.Environment]::GetEnvironmentVariable('CARGO') || "cargo"
    if (!(Get-Command "$Cargo" -errorAction SilentlyContinue)) {
        Write-Error "FATAL: -Cargo flag was not set to a valid 'cargo' binary"
        exit 1
    }

    # create .result directory as the release workflow requires it
    New-Item -Path . -Name ".result" -ItemType Directory

    $BuildFlags = [System.Environment]::GetEnvironmentVariable('BUILDFLAGS')

    Write-Host "$ $Cargo build --release --locked $BuildFlags"
    iex "$Cargo build --release --locked $BuildFlags"
    if (!$?) {
        Write-Error "Failed to run 'cargo build', exiting early"
        exit 1
    }

    # Move ./target/release/ume.exe ~> ./.result/ume.exe
    Move-Item -Path "./target/release/ume.exe" -Destination "./.result/ume-windows-x86_64.exe"

    Push-Location ./.result
    (Get-FileHash -Path "ume-windows-x86_64.exe").Hash.ToLower() | Out-File "ume-windows-x86_64.exe.sha256"

    Pop-Location

    Write-Host "Completed."
}

Main
