<#
.SYNOPSIS
    Builds MetroMind from source and installs it for the current user.

.DESCRIPTION
    Runs a release build with cargo, copies the executable to the install folder,
    adds a Start Menu shortcut, and registers the .metromind file type so map files
    open in MetroMind when double-clicked. Everything is per-user: no administrator
    rights are needed. Run it again to update an existing install.

.PARAMETER InstallDir
    Where MetroMind.exe goes. Defaults to %LOCALAPPDATA%\Programs\MetroMind.

.PARAMETER DesktopShortcut
    Also put a shortcut on the desktop.

.PARAMETER NoFileAssociation
    Don't register the .metromind file type.

.PARAMETER RunTests
    Run the test suite before installing, and stop if it fails.

.PARAMETER Uninstall
    Remove the executable, shortcuts and file association. Map files and app
    settings are kept.

.EXAMPLE
    .\scripts\install.ps1

.EXAMPLE
    .\scripts\install.ps1 -RunTests -DesktopShortcut

.EXAMPLE
    .\scripts\install.ps1 -Uninstall
#>
[CmdletBinding()]
param(
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'Programs\MetroMind'),
    [switch]$DesktopShortcut,
    [switch]$NoFileAssociation,
    [switch]$RunTests,
    [switch]$Uninstall
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$AppName = 'MetroMind'
$ExeName = 'MetroMind.exe'
$Package = 'metromind-app'
$RepoRoot = Split-Path -Parent $PSScriptRoot
$InstalledExe = Join-Path $InstallDir $ExeName
$StartMenuLink = Join-Path ([Environment]::GetFolderPath('Programs')) "$AppName.lnk"
$DesktopLink = Join-Path ([Environment]::GetFolderPath('Desktop')) "$AppName.lnk"
$ProgId = 'MetroMind.Map'
$ClassesKey = 'HKCU:\Software\Classes'

function Write-Step([string]$Message) {
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Invoke-Cargo([string[]]$Arguments) {
    & cargo @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "cargo $($Arguments -join ' ') failed with exit code $LASTEXITCODE."
    }
}

function Stop-RunningApp {
    # A running copy locks the exe, so the copy would fail.
    $running = Get-Process -ErrorAction SilentlyContinue |
        Where-Object { $_.Path -and $_.Path -ieq $InstalledExe }
    if ($running) {
        Write-Step "Closing the running $AppName (it autosaves on exit)"
        $running | ForEach-Object { [void]$_.CloseMainWindow() }
        $running | Wait-Process -Timeout 10 -ErrorAction SilentlyContinue
        $running | Where-Object { -not $_.HasExited } | Stop-Process -Force
    }
}

function New-Shortcut([string]$Path) {
    $shell = New-Object -ComObject WScript.Shell
    $link = $shell.CreateShortcut($Path)
    $link.TargetPath = $InstalledExe
    $link.WorkingDirectory = $InstallDir
    $link.Description = 'Brainstorm ideas on a metro map'
    $link.Save()
}

function Register-FileType {
    $command = "`"$InstalledExe`" `"%1`""
    New-Item -Path "$ClassesKey\.metromind" -Value $ProgId -Force | Out-Null
    New-Item -Path "$ClassesKey\$ProgId" -Value 'MetroMind map' -Force | Out-Null
    New-Item -Path "$ClassesKey\$ProgId\DefaultIcon" -Value "`"$InstalledExe`",0" -Force | Out-Null
    New-Item -Path "$ClassesKey\$ProgId\shell\open\command" -Value $command -Force | Out-Null
    Update-ShellAssociations
}

function Unregister-FileType {
    foreach ($key in "$ClassesKey\.metromind", "$ClassesKey\$ProgId") {
        if (Test-Path $key) { Remove-Item -Path $key -Recurse -Force }
    }
    Update-ShellAssociations
}

function Update-ShellAssociations {
    # Tell Explorer the associations changed so icons refresh without a sign-out.
    if (-not ('MetroMind.Shell' -as [type])) {
        Add-Type -Namespace MetroMind -Name Shell -MemberDefinition @'
[System.Runtime.InteropServices.DllImport("shell32.dll")]
public static extern void SHChangeNotify(int eventId, uint flags, System.IntPtr item1, System.IntPtr item2);
'@
    }
    [MetroMind.Shell]::SHChangeNotify(0x08000000, 0, [IntPtr]::Zero, [IntPtr]::Zero)
}

function Uninstall-App {
    Stop-RunningApp
    Write-Step "Removing $AppName"
    foreach ($path in $StartMenuLink, $DesktopLink) {
        if (Test-Path $path) { Remove-Item $path -Force }
    }
    Unregister-FileType
    if (Test-Path $InstallDir) { Remove-Item $InstallDir -Recurse -Force }
    Write-Host "$AppName is uninstalled. Your maps and settings were left in place." -ForegroundColor Green
}

function Install-App {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw 'cargo was not found. Install Rust from https://rustup.rs and open a new terminal.'
    }

    Push-Location $RepoRoot
    try {
        if ($RunTests) {
            Write-Step 'Running tests'
            Invoke-Cargo @('test', '--workspace', '--release')
        }
        Write-Step 'Building release binary'
        Invoke-Cargo @('build', '--release', '--locked', '-p', $Package)
    }
    finally {
        Pop-Location
    }

    $builtExe = Join-Path $RepoRoot "target\release\$Package.exe"
    if (-not (Test-Path $builtExe)) {
        throw "The build finished but $builtExe is missing."
    }

    Stop-RunningApp
    Write-Step "Installing to $InstallDir"
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Copy-Item $builtExe $InstalledExe -Force

    Write-Step 'Adding Start Menu shortcut'
    New-Shortcut $StartMenuLink
    if ($DesktopShortcut) {
        Write-Step 'Adding desktop shortcut'
        New-Shortcut $DesktopLink
    }

    if ($NoFileAssociation) {
        Unregister-FileType
    }
    else {
        Write-Step 'Registering .metromind files'
        Register-FileType
    }

    Write-Host "$AppName is installed. Start it from the Start Menu or run:" -ForegroundColor Green
    Write-Host "  & `"$InstalledExe`""
}

if ($Uninstall) { Uninstall-App } else { Install-App }
