$ErrorActionPreference = "Continue"
$app = Join-Path $PWD "src-tauri\target\debug\converter.exe"
$out = Join-Path $PWD "e2e-debug"
New-Item -ItemType Directory -Force $out | Out-Null

Write-Host "== WebView2 runtime"
Get-ItemProperty "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" -ErrorAction SilentlyContinue | Select-Object pv, location | Format-List
Write-Host "== msedgedriver"
Get-Command msedgedriver.exe | Format-List Source
& msedgedriver.exe --version
Write-Host "== WEBVIEW2 env"
Get-ChildItem env: | Where-Object Name -like "WEBVIEW2*" | Format-List
Write-Host "== Session"
whoami
Get-Process -Id $PID | Select-Object SessionId
$identity = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
Write-Host "Elevated=$($identity.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator))"
whoami /groups | Select-String "Mandatory Label"
Write-Host "== WebView2 policy"
Get-ItemProperty "HKLM:\SOFTWARE\Policies\Microsoft\Edge\WebView2\AdditionalBrowserArguments" -ErrorAction SilentlyContinue | Format-List

function Show-Processes($label) {
  Write-Host "== Processes: $label"
  Get-CimInstance Win32_Process | Where-Object { $_.Name -in @("converter.exe", "msedgewebview2.exe", "msedgedriver.exe") } |
    ForEach-Object { "{0} {1} parent={2}: {3}" -f $_.Name, $_.ProcessId, $_.ParentProcessId, $_.CommandLine } | Out-File -Append -Width 4000 "$out\processes.txt"
  Get-CimInstance Win32_Process -Filter "name='msedgewebview2.exe'" | Where-Object { $_.CommandLine -notmatch "--type=" } | ForEach-Object { $_.CommandLine -split " --" } | Write-Host
}

Write-Host "== Launch the app directly"
$proc = Start-Process -FilePath $app -PassThru -RedirectStandardOutput "$out\app-stdout.txt" -RedirectStandardError "$out\app-stderr.txt"
Start-Sleep -Seconds 20
Write-Host "HasExited=$($proc.HasExited) ExitCode=$($proc.ExitCode)"
Show-Processes "direct launch"
Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
Get-Process msedgewebview2 -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 3
Write-Host "-- stdout"; Get-Content "$out\app-stdout.txt"
Write-Host "-- stderr"; Get-Content "$out\app-stderr.txt"

Write-Host "== Direct msedgedriver session"
$driver = Start-Process -FilePath msedgedriver.exe -ArgumentList "--port=9515", "--verbose", "--log-path=$out\msedgedriver.log" -PassThru
Start-Sleep -Seconds 3
$body = @{ capabilities = @{ alwaysMatch = @{ browserName = "webview2"; "ms:edgeChromium" = $true; "ms:edgeOptions" = @{ binary = $app; args = @() } } } } | ConvertTo-Json -Depth 6
$job = Start-Job -ScriptBlock {
  param($body)
  try { Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:9515/session" -Body $body -ContentType "application/json" -TimeoutSec 120 | ConvertTo-Json -Depth 6 }
  catch { "FAILED: $($_.Exception.Message) $($_.ErrorDetails.Message)" }
} -ArgumentList $body
Start-Sleep -Seconds 20
Show-Processes "during session"
Write-Host "-- user data folders"
Get-ChildItem -Recurse -Depth 2 "$env:LOCALAPPDATA\com.squ1ggly.converter" -ErrorAction SilentlyContinue | Select-Object FullName | Format-Table -AutoSize | Out-String -Width 300 | Write-Host
Get-ChildItem -Recurse -Filter DevToolsActivePort "$env:LOCALAPPDATA", "$env:TEMP", "C:\Windows\SystemTemp" -ErrorAction SilentlyContinue | Select-Object FullName | Format-Table -AutoSize | Out-String -Width 300 | Write-Host
Receive-Job $job -Wait | Write-Host
Show-Processes "after session"
Stop-Process -Id $driver.Id -Force -ErrorAction SilentlyContinue
Get-Process converter, msedgewebview2 -ErrorAction SilentlyContinue | Stop-Process -Force
Write-Host "-- msedgedriver log (filtered)"
Get-Content "$out\msedgedriver.log" | Select-Object -First 200
exit 0
