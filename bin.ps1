param(
    [string]$Target,
    [switch]$r
)

$ErrorActionPreference = "Stop"

$Bins = @{
    "http"      = "http-server"
    "rpc"       = "rpc-server"
    "inspector" = "inspector"
    "master"    = "update_master"
}

if (-not $Target -or -not $Bins.ContainsKey($Target)) {
    Write-Host "usage: ./bin <http|rpc|master|inspector> [-r]"
    exit 1
}

if ($Target -eq "master") {
    $cargoArgs = @("run", "-p", "resource", "--bin", "update_master")
    if ($r) {
        $cargoArgs += "--release"
    }
    & cargo @cargoArgs
    exit $LASTEXITCODE
}

$ConfigPath = Join-Path $PSScriptRoot "config.toml"
if (Test-Path $ConfigPath) {
    $Level = if ($r) { "info" } else { "debug" }
    $Text = Get-Content $ConfigPath -Raw
    $Text = $Text -replace '(?m)^([ \t]*level[ \t]*=[ \t]*")[^"]*(")', ('${1}' + $Level + '${2}')
    Set-Content -Path $ConfigPath -Value $Text -NoNewline
}
else {
    Write-Warning "config.toml not found; leaving log level untouched"
}

$cargoArgs = @(
    "run"
    "--bin"
    $Bins[$Target]
)

if ($r) {
    $cargoArgs += "--release"
}

if ($args.Count -gt 0) {
    $cargoArgs += "--"
    $cargoArgs += $args
}

& cargo @cargoArgs
