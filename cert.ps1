$CertDir    = Join-Path $PSScriptRoot "resource\cert"
$CertFile   = Join-Path $CertDir "cert.pem"
$KeyFile    = Join-Path $CertDir "key.pem"
$CommonName = "127.0.0.1"
$Days       = 365

$IsAdmin = ([Security.Principal.WindowsPrincipal] `
    [Security.Principal.WindowsIdentity]::GetCurrent()
).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $IsAdmin) {
    Start-Process powershell `
        -Verb RunAs `
        -ArgumentList "-ExecutionPolicy Bypass -File `"$PSCommandPath`""
    exit
}

$OpenSSL = Get-Command openssl -ErrorAction SilentlyContinue

if (-not $OpenSSL) {
    Write-Error "openssl not found, you can get it from https://slproweb.com/products/Win32OpenSSL.html"
    exit 1
}

New-Item -ItemType Directory -Force -Path $CertDir | Out-Null

if (Test-Path $CertFile) {
    try {
        $Thumbprint = (Get-PfxCertificate $CertFile).Thumbprint

        Get-ChildItem Cert:\LocalMachine\Root |
            Where-Object Thumbprint -eq $Thumbprint |
            Remove-Item -Force

        Write-Host "removed existing cert from trusted root"
    }
    catch {
        Write-Host "existing cert was not trusted?"
    }
}

& $OpenSSL.Source req `
    -x509 `
    -newkey rsa:2048 `
    -nodes `
    -keyout $KeyFile `
    -out $CertFile `
    -days $Days `
    -subj "/CN=$CommonName"

if ($LASTEXITCODE -ne 0) {
    throw "openssl failed to generate the cert"
}

Import-Certificate `
    -FilePath $CertFile `
    -CertStoreLocation Cert:\LocalMachine\Root | Out-Null

Write-Host ""
Write-Host "success!"
Write-Host "cert: $CertFile"
Write-Host "priv key: $KeyFile"
