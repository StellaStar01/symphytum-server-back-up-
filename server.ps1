param(
    [switch]$r
)

$ErrorActionPreference = "Stop"

$cargoArgs = @(
    "run"
    "--bin"
    "server"
)

if ($r) {
    $cargoArgs += "--release"
}

if ($args.Count -gt 0) {
    $cargoArgs += "--"
    $cargoArgs += $args
}

& cargo @cargoArgs
