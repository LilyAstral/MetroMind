<#
.SYNOPSIS
    Builds a portable MetroMind.exe and zips it for release.

.DESCRIPTION
    Runs a release build and writes MetroMind-windows.zip, holding MetroMind.exe
    with its icon embedded. The exe runs from anywhere; maps and settings live in
    %APPDATA%\MetroMind. It is not code-signed, so SmartScreen may warn on first
    launch (More info -> Run anyway). To install with shortcuts and the .metromind
    file type instead, use scripts\install.ps1.

.PARAMETER OutDir
    Where the zip goes. Defaults to dist\windows.

.EXAMPLE
    .\scripts\bundle-windows.ps1
#>
[CmdletBinding()]
param(
    [string]$OutDir = 'dist\windows'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RepoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $RepoRoot
try {
    Write-Host '==> Building release binary' -ForegroundColor Cyan
    & cargo build --release --locked -p metromind-app
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed with exit code $LASTEXITCODE." }

    Write-Host "==> Packaging into $OutDir" -ForegroundColor Cyan
    $stage = Join-Path $OutDir 'MetroMind'
    $zip = Join-Path $OutDir 'MetroMind-windows.zip'
    if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
    New-Item -ItemType Directory -Force $stage | Out-Null
    Copy-Item 'target\release\metromind-app.exe' (Join-Path $stage 'MetroMind.exe')
    Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip -Force
    Remove-Item $stage -Recurse -Force

    Write-Host "Done: $zip" -ForegroundColor Green
}
finally {
    Pop-Location
}
