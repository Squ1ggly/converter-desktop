# Writes a Tauri config overlay that signs the Windows installers, and points the build at it
# through WINDOWS_SIGNING_CONFIG. Uses an OV certificate when WINDOWS_CERTIFICATE is set, or
# Azure Artifact Signing when AZURE_CLIENT_ID is set. Without either, the build stays unsigned.
$ErrorActionPreference = "Stop"
$configPath = Join-Path $env:RUNNER_TEMP "windows-signing.json"

function Import-Certificate {
  $certificatePath = Join-Path $env:RUNNER_TEMP "certificate.pfx"
  [System.IO.File]::WriteAllBytes($certificatePath, [System.Convert]::FromBase64String($env:WINDOWS_CERTIFICATE))
  $password = ConvertTo-SecureString $env:WINDOWS_CERTIFICATE_PASSWORD -AsPlainText -Force
  $certificate = Import-PfxCertificate -FilePath $certificatePath -CertStoreLocation Cert:\CurrentUser\My -Password $password
  Remove-Item $certificatePath
  return $certificate.Thumbprint
}

if ($env:WINDOWS_CERTIFICATE) {
  $thumbprint = Import-Certificate
  $windowsConfig = @{
    certificateThumbprint = $thumbprint
    digestAlgorithm = "sha256"
    timestampUrl = $env:WINDOWS_TIMESTAMP_URL
  }
  Write-Output "Signing with the certificate $thumbprint"
} elseif ($env:AZURE_CLIENT_ID) {
  cargo install trusted-signing-cli --locked
  $windowsConfig = @{
    signCommand = @{
      cmd = "trusted-signing-cli"
      args = @("-e", $env:ARTIFACT_SIGNING_ENDPOINT, "-a", $env:ARTIFACT_SIGNING_ACCOUNT, "-c", $env:ARTIFACT_SIGNING_PROFILE, "-d", "Converter", "%1")
    }
  }
  Write-Output "Signing with Azure Artifact Signing account $env:ARTIFACT_SIGNING_ACCOUNT"
} else {
  Write-Output "No Windows signing secrets are set, so the installers will be unsigned."
  return
}

@{ bundle = @{ windows = $windowsConfig } } | ConvertTo-Json -Depth 5 | Set-Content -Path $configPath -Encoding utf8
"WINDOWS_SIGNING_CONFIG=$configPath" >> $env:GITHUB_ENV
