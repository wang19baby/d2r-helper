# verify-pi-env.ps1
# Run in Windows PowerShell (NOT WSL bash) to verify Pi setup.
# Usage: cd to D2R repo, then powershell -ExecutionPolicy Bypass -File tools\verify-pi-env.ps1
$ErrorActionPreference = "Continue"
Write-Host "==== verify-pi-env.ps1 ====" -ForegroundColor Cyan

# 1. qmd setup
Write-Host "`n[1] qmd version" -ForegroundColor Yellow
$qmd = (Get-Command qmd -ErrorAction SilentlyContinue)
if ($qmd) {
  & qmd --version
  Write-Host "`n[1b] qmd collection add" -ForegroundColor Yellow
  & qmd collection add $env:USERPROFILE\.pi\agent\memory --name pi-memory 2>&1 | Select-Object -First 5
  if ($LASTEXITCODE -eq 0) {
    Write-Host "`n[1c] qmd embed" -ForegroundColor Yellow
    & qmd embed 2>&1 | Select-Object -First 20
  }
} else {
  Write-Host "qmd NOT found on PATH — install with: npm install -g @tobilu/qmd" -ForegroundColor Red
}

# 2. memory_search smoke test
Write-Host "`n[2] memory_search smoke test (qmd)" -ForegroundColor Yellow
& qmd search "d2r-marketplace-tauri" --limit 3 2>&1 | Select-Object -First 20

# 3. rust-analyzer
Write-Host "`n[3] rust-analyzer --version" -ForegroundColor Yellow
$ra = (Get-Command rust-analyzer -ErrorAction SilentlyContinue)
if ($ra) {
  & rust-analyzer --version
} else {
  Write-Host "rust-analyzer NOT found on PATH — check $env:USERPROFILE\.cargo\bin" -ForegroundColor Red
}

# 4. cargo check inside src-tauri (this is the real LSP smoke test)
Write-Host "`n[4] cargo check (src-tauri, ~5-10 min first run)" -ForegroundColor Yellow
$tauriDir = Join-Path $PSScriptRoot "..\src-tauri"
if (Test-Path $tauriDir) {
  Push-Location $tauriDir
  & cargo check --message-format=short 2>&1 | Select-Object -Last 40
  Pop-Location
} else {
  Write-Host "src-tauri NOT found at $tauriDir" -ForegroundColor Red
}

# 5. existing rust-analyzer config check
Write-Host "`n[5] .pi/pi-lsp.json" -ForegroundColor Yellow
$lspCfg = Join-Path $PSScriptRoot "..\.pi\pi-lsp.json"
if (Test-Path $lspCfg) {
  Get-Content $lspCfg | Write-Host
} else {
  Write-Host "missing — recreate from memory" -ForegroundColor Red
}

Write-Host "`n==== DONE ====" -ForegroundColor Cyan
