(Get-ChildItem Cert:\LocalMachine\Root | Where-Object Thumbprint -eq (Get-PfxCertificate .\cert.pem).Thumbprint) | Remove-Item

openssl req -x509 -newkey rsa:2048 -nodes -keyout key.pem -out cert.pem -days 365 -config server.cnf -extensions v3_req

Import-Certificate -FilePath .\cert.pem -CertStoreLocation Cert:\LocalMachine\Root
