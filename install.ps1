#Requires -Version 5.1

$ErrorActionPreference = 'Stop'

$sourcePath = Join-Path $PSScriptRoot 'glazewm-window-info.ps1'
$installDirectory = Join-Path $env:LOCALAPPDATA 'GlazeWMWindowInfo'
$installedPath = Join-Path $installDirectory 'glazewm-window-info.ps1'

if (-not (Test-Path -LiteralPath $sourcePath -PathType Leaf)) {
    throw "Source script was not found: $sourcePath"
}

New-Item -ItemType Directory -Path $installDirectory -Force | Out-Null
Copy-Item -LiteralPath $sourcePath -Destination $installedPath -Force

Write-Host "Installed GlazeWM Window Info to:"
Write-Host $installedPath
