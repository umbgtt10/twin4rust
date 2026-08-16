# Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
# Licensed under the MIT License
# SPDX-License-Identifier: MIT

$ErrorActionPreference = "Stop"
Push-Location (Split-Path $PSScriptRoot -Parent)

function Invoke-Crap4RustGate {
    param(
        [string]$Label,
        [string[]]$Packages,
        [double]$Threshold = 15,
        [string[]]$ExcludePaths = @()
    )

    Write-Host "$Label..." -ForegroundColor Cyan

    $manifestPath = (Resolve-Path (Join-Path $PSScriptRoot "..\Cargo.toml")).Path
    $args = @("--manifest-path", $manifestPath)
    foreach ($package in $Packages) {
        $args += @("--package", $package)
    }
    foreach ($excludePath in $ExcludePaths) {
        $args += @("--exclude-path", $excludePath)
    }
    $args += @("--warn-only", "--threshold", $Threshold.ToString())

    $previousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $output = & cargo crap4rust @args 2>&1
    $ErrorActionPreference = $previousErrorActionPreference
    $exitCode = $LASTEXITCODE
    $output | ForEach-Object { Write-Host $_ }

    if ($exitCode -ne 0) {
        Write-Host "`nFailed: $Label (exit code $exitCode)" -ForegroundColor Red
        Pop-Location
        exit 1
    }

    $summaryLine = $output | Select-String -Pattern "summary:\s+total_functions=.*crappy_functions=(\d+)"
    if (-not $summaryLine) {
        Write-Host "`nFailed: $Label (could not parse crap4rust summary)" -ForegroundColor Red
        Pop-Location
        exit 1
    }

    $crappyCount = [int]$summaryLine.Matches[0].Groups[1].Value
    if ($crappyCount -gt 0) {
        Write-Host "`nFailed: $Label ($crappyCount crappy functions detected)" -ForegroundColor Red
        Pop-Location
        exit 1
    }
}

# A tool that enforces a rule it does not itself satisfy is not worth
# installing. This runs the freshly built binary rather than whatever version
# happens to be installed, so the gate reflects the working tree.
function Invoke-Twin4RustSelfGate {
    param([string]$Label = "twin4rust self-analysis")

    Write-Host "$Label..." -ForegroundColor Cyan

    $manifestPath = (Resolve-Path (Join-Path $PSScriptRoot "..\Cargo.toml")).Path

    $previousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    & cargo run --quiet --bin cargo-twin4rust -- --manifest-path $manifestPath
    $exitCode = $LASTEXITCODE
    $ErrorActionPreference = $previousErrorActionPreference

    if ($exitCode -ne 0) {
        Write-Host "`nFailed: $Label (exit code $exitCode)" -ForegroundColor Red
        Pop-Location
        exit 1
    }
}

# ---------------------------------------------------------------------------
# CRAP gate
# ---------------------------------------------------------------------------

Invoke-Crap4RustGate "CRAP twin4rust" @("cargo-twin4rust")

# ---------------------------------------------------------------------------
# Mirrored test gate (self-analysis)
# ---------------------------------------------------------------------------

Invoke-Twin4RustSelfGate

# ---------------------------------------------------------------------------

Write-Host "`ntwin4rust Stage 2 passed!" -ForegroundColor Green
Pop-Location
exit 0
