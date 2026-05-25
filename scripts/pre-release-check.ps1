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

function Get-ReleaseVersionFromBundle {
  param([string]$BundleVersion)

  if ($BundleVersion -notmatch '^0\.0\.(\d{2,})$') {
    throw "번들 버전은 0.0.xx 형식이어야 합니다: $BundleVersion"
  }

  $compactPatch = $Matches[1]
  return "0.0.$($compactPatch.Substring(0, $compactPatch.Length - 1)).$($compactPatch.Substring($compactPatch.Length - 1))"
}

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

  $tauriConfigPath = Join-Path $configManager "src-tauri\tauri.conf.json"
  $packagePath = Join-Path $configManager "package.json"
  $cargoPath = Join-Path $configManager "src-tauri\Cargo.toml"
  $tauriConfig = Get-Content -Raw -Encoding UTF8 $tauriConfigPath | ConvertFrom-Json
  $targets = @($tauriConfig.bundle.targets)
  if ($targets -contains "msi" -or $targets -contains "all") {
    throw "릴리즈 빌드 대상에 MSI가 포함되어 있습니다. bundle.targets는 nsis만 허용합니다."
  }

  $packageJson = Get-Content -Raw -Encoding UTF8 $packagePath | ConvertFrom-Json
  $releaseVersion = if ($Tag) { $Tag.TrimStart("v") } else { Get-ReleaseVersionFromBundle -BundleVersion $packageJson.version }
  if ($releaseVersion -notmatch '^0\.0\.(\d+)\.(\d+)$') {
    throw "앱 버전은 0.0.x.x 형식이어야 합니다: $releaseVersion"
  }
  $bundleVersion = "0.0.$($Matches[1])$($Matches[2])"
  $cargoToml = Get-Content -Raw -Encoding UTF8 $cargoPath
  if ($packageJson.version -ne $bundleVersion) {
    throw "package.json 버전($($packageJson.version))이 앱 릴리즈 버전($releaseVersion)의 번들 버전($bundleVersion)과 다릅니다."
  }
  if ($tauriConfig.version -ne $bundleVersion) {
    throw "tauri.conf.json 버전($($tauriConfig.version))이 번들 버전($bundleVersion)과 다릅니다."
  }
  if ($cargoToml -notmatch "(?m)^version\s*=\s*`"$([regex]::Escape($bundleVersion))`"") {
    throw "Cargo.toml 버전이 번들 버전($bundleVersion)과 다릅니다."
  }

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
