param(
    [string]$Target,
    [switch]$r
)

$ErrorActionPreference = "Stop"

if (-not $Target -or ($Target -ne "rpc" -and $Target -ne "inspector")) {
    Write-Host "usage: .\bin.ps1 <rpc|inspector> [-r]"
    exit 1
}

$ConfigPath = Join-Path $PSScriptRoot "config.toml"
if (Test-Path $ConfigPath) {
    $Level = if ($r) { "info" } else { "debug" }
    $Text = Get-Content $ConfigPath -Raw
    $Text = $Text -replace '(?m)^([ \t]*level[ \t]*=[ \t]*")[^"]*(")', ('${1}' + $Level + '${2}')
    Set-Content -Path $ConfigPath -Value $Text -NoNewline
    # Write-Host "log level set to $Level in config.toml"
}
else {
    Write-Warning "config.toml not found; leaving log level untouched"
}

$cargoArgs = @(
    "run"
    "--bin"
    $Target
)

if ($r) {
    $cargoArgs += "--release"
}

if ($args.Count -gt 0) {
    $cargoArgs += "--"
    $cargoArgs += $args
}

& cargo @cargoArgs
