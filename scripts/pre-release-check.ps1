param(
  [string]$Repository = "minseokk77/DS5Dongle",
  [string]$Tag = "",
  [switch]$VerifyReleaseSource
)

$ErrorActionPreference = "Stop"

function Require-Command {
  param([string]$Name)
  if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
    throw "$Name 명령을 찾지 못했습니다."
  }
}

Require-Command git
Require-Command pnpm.cmd
Require-Command cargo
Require-Command rg

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$configManager = Join-Path $repoRoot "config-manager"

Push-Location $repoRoot
try {
  Push-Location $configManager
  try {
    pnpm.cmd build
  }
  finally {
    Pop-Location
  }

  cargo check --manifest-path ".\config-manager\src-tauri\Cargo.toml"

  $mojibakePattern = [string]::Join("|", @([char]0xfffd, [char]0x8adb, [char]0xc891, [char]0xc7fe, [char]0xb5e3))
  $mojibake = rg -n $mojibakePattern config-manager\src config-manager\src-tauri --glob "!config-manager/src-tauri/target/**"
  if ($LASTEXITCODE -eq 0) {
    throw "한글 깨짐 의심 패턴이 발견되었습니다.`n$mojibake"
  }
  if ($LASTEXITCODE -gt 1) {
    throw "한글 깨짐 검사 실행에 실패했습니다."
  }

  Get-ChildItem -LiteralPath (Join-Path $repoRoot "scripts") -Filter "*.ps1" -File | ForEach-Object {
    $errors = $null
    [System.Management.Automation.PSParser]::Tokenize((Get-Content -Raw -Encoding UTF8 $_.FullName), [ref]$errors) | Out-Null
    if ($errors) {
      throw "PowerShell 문법 오류: $($_.Name) - $($errors[0].Message)"
    }
  }

  if ($VerifyReleaseSource) {
    if (-not $Tag) {
      throw "릴리즈 소스 검증에는 -Tag 값이 필요합니다."
    }
    & (Join-Path $repoRoot "scripts\verify-release-source.ps1") -Tag $Tag -Repository $Repository
  }

  Write-Host "Pre-release checks passed."
}
finally {
  Pop-Location
}
