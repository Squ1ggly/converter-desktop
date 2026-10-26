# Installs Converter silently from an NSIS or MSI installer, checks the installed files and, for
# signed builds, their signatures, runs FFmpeg, launches the app, then uninstalls it and checks the
# files are gone.
#
#   pwsh scripts/smoke-tests/windows.ps1 <installer>
param([Parameter(Mandatory)] [string] $InstallerPath)

$ErrorActionPreference = "Stop"
$installDirectory = Join-Path $env:ProgramFiles "Converter"
$appPath = Join-Path $installDirectory "converter.exe"
$ffmpegPath = Join-Path $installDirectory "converter-ffmpeg.exe"
$isMsi = $InstallerPath.EndsWith(".msi")
$msiLogPath = Join-Path $env:RUNNER_TEMP "msiexec.log"
$launchWaitSeconds = 15
$uninstallTimeoutSeconds = 120

# PE machine types: https://learn.microsoft.com/windows/win32/debug/pe-format#machine-types
$machineNames = @{ 0x8664 = "x64"; 0xAA64 = "arm64" }

function Fail([string] $message) {
  throw "FAIL: $message"
}

function Get-MachineName([string] $executablePath) {
  $bytes = [System.IO.File]::ReadAllBytes($executablePath)
  $peHeaderOffset = [System.BitConverter]::ToInt32($bytes, 0x3C)
  $machine = [System.BitConverter]::ToUInt16($bytes, $peHeaderOffset + 4)
  $machineName = $machineNames[[int] $machine]
  return $machineName
}

function Invoke-Msiexec([string[]] $arguments) {
  $process = Start-Process msiexec.exe -ArgumentList ($arguments + @("/qn", "/l*v", "`"$msiLogPath`"")) -Wait -PassThru
  if ($process.ExitCode -ne 0) {
    Get-Content $msiLogPath -Tail 60
    Fail "msiexec $($arguments[0]) exited with $($process.ExitCode)"
  }
}

function Install-Converter {
  if ($isMsi) {
    Invoke-Msiexec @("/i", "`"$InstallerPath`"")
    return
  }

  $process = Start-Process $InstallerPath -ArgumentList "/S" -Wait -PassThru
  if ($process.ExitCode -ne 0) {
    Fail "The installer exited with $($process.ExitCode)"
  }
}

function Uninstall-Converter {
  if ($isMsi) {
    Invoke-Msiexec @("/x", "`"$InstallerPath`"")
    return
  }

  # The NSIS uninstaller copies itself to a temporary folder and returns at once.
  Start-Process (Join-Path $installDirectory "uninstall.exe") -ArgumentList "/S" -Wait
  $deadline = (Get-Date).AddSeconds($uninstallTimeoutSeconds)
  while ((Test-Path $appPath) -and ((Get-Date) -lt $deadline)) {
    Start-Sleep -Seconds 2
  }
}

function Assert-InstalledFiles([string] $expectedMachine) {
  foreach ($relativePath in @("converter.exe", "converter-ffmpeg.exe", "LICENSE", "THIRD-PARTY-NOTICES.md", "licenses\FFmpeg\COPYING.GPLv3", "licenses\FFmpeg\LICENSE.md")) {
    $filePath = Join-Path $installDirectory $relativePath
    if (-not (Test-Path $filePath -PathType Leaf)) {
      Fail "$filePath is missing"
    }
  }

  foreach ($executablePath in @($appPath, $ffmpegPath)) {
    $machineName = Get-MachineName $executablePath
    if ($machineName -ne $expectedMachine) {
      Fail "$executablePath is built for $machineName, expected $expectedMachine"
    }
  }
  Write-Output "ok   installed files are present and built for $expectedMachine"
}

# A signed installer must install a signed app. The bundled FFmpeg comes unsigned from upstream.
function Assert-Signatures {
  $installerSignature = Get-AuthenticodeSignature $InstallerPath
  if ($installerSignature.Status -eq "NotSigned") {
    Write-Output "ok   the installer is unsigned"
    return
  }

  foreach ($signedPath in @($InstallerPath, $appPath)) {
    $signature = Get-AuthenticodeSignature $signedPath
    if ($signature.Status -ne "Valid") {
      Fail "The signature of $signedPath is $($signature.Status): $($signature.StatusMessage)"
    }
  }
  Write-Output "ok   the installer and the app are signed by $($installerSignature.SignerCertificate.Subject)"
}

function Assert-FfmpegWorks {
  $versionOutput = & $ffmpegPath -hide_banner -version
  if ($LASTEXITCODE -ne 0) {
    Fail "converter-ffmpeg -version exited with $LASTEXITCODE"
  }
  Write-Output ($versionOutput | Select-Object -First 1)

  $gitBash = Join-Path $env:ProgramFiles "Git\bin\bash.exe"
  & $gitBash (Join-Path $PSScriptRoot "ffmpeg.sh") ($ffmpegPath -replace "\\", "/")
  if ($LASTEXITCODE -ne 0) {
    Fail "The FFmpeg conversion checks failed"
  }
}

# WebView2 runs in msedgewebview2.exe processes that use the app's own data folder, so finding
# one proves the window's web view started.
function Assert-AppLaunches {
  $process = Start-Process $appPath -PassThru
  Start-Sleep -Seconds $launchWaitSeconds
  if ($process.HasExited) {
    Fail "Converter exited with $($process.ExitCode) within $launchWaitSeconds seconds"
  }

  $webViewProcesses = Get-CimInstance Win32_Process -Filter "Name = 'msedgewebview2.exe'" |
    Where-Object { $_.CommandLine -like "*com.squ1ggly.converter*" }
  if (-not $webViewProcesses) {
    Fail "Converter is running but its WebView2 did not start"
  }
  Write-Output "ok   Converter and its WebView2 are running after $launchWaitSeconds seconds"

  Stop-Process -Id $process.Id
  $process.WaitForExit()
}

$installerName = Split-Path $InstallerPath -Leaf
$expectedMachine = if ($installerName -match "_arm64") { "arm64" } else { "x64" }
Write-Output "Testing $installerName on $env:PROCESSOR_ARCHITECTURE Windows"

Install-Converter
Assert-InstalledFiles $expectedMachine
Assert-Signatures
Assert-FfmpegWorks
Assert-AppLaunches
Uninstall-Converter

if (Test-Path $appPath) {
  Fail "$appPath is still there after uninstalling"
}
Write-Output "ok   uninstalled"
Write-Output "All Windows checks passed for $installerName"
