# vendored from:
# https://github.com/charted-dev/charted/blob/fd8ee07a34bb15e3bee3f3f74d5736c460a83154/src/ci/windows/Release.ps1

# 🐻‍❄️📦 charted-server: Free, open source, and reliable Helm Chart registry made in Rust
# Copyright 2022-2025 Noelware, LLC. <team@noelware.org>
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#    http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

Param(
    [Parameter()]
    [string]$Cargo = "cargo",

    [Parameter()]
    [string]$BuildFlags = ""
)

$ErrorActionPreference = "Stop"
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass -Force

. "$PSScriptRoot\..\_shared.ps1"

function Main {
    Param(
        [Parameter()]
        [string]$Binary
    )

    if (![System.Environment]::Is64BitOperatingSystem) {
        Write-Error ">>> ume is not supported on 32-bit systems"
        Exit 1
    }

    StartGroup "Build / Windows (x64)"

    $CargoFlags = [System.Environment]::GetEnvironmentVariable("CARGOFLAGS") || ""

    # Create the `.result` directory in the root project
    # so we can put our files in there.
    New-Item -Path . -Name ".result" -ItemType Directory || true

    Write-Host "$ $Cargo $CargoFlags build --release --locked --bin $Binary $BuildFlags"
    Invoke-Expression "$Cargo $CargoFlags build --release --locked --bin $Binary $BuildFlags"

    Write-Host "$ mv ./target/release/$Binary.exe -> .result/$Binary-windows-x86_64.exe"
    Move-Item -Path "./target/release/$Binary.exe" ".result/$Binary-windows-x86_64.exe"

    Push-Location ".result"

    Write-Host "$ Compute checksum of binary"

    $Hash = (Get-FileHash -Path "$Binary-windows-x86_64.exe").Hash.ToLower()
    Write-Output "$Hash  $Binary-windows-x86_64.exe" | Out-File "$Binary-windows-x86_64.exe.sha256"

    Pop-Location

    Write-Host "$ Completed every task. All resources are in .result!"

    EndGroup
}

Main "ume"
