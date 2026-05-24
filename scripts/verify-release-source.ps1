param(
  [Parameter(Mandatory = $true)]
  [string]$Tag,

  [string]$Repository = "minseokk77/DS5Dongle"
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
  throw "GitHub CLI(gh)를 찾지 못했습니다."
}

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
  throw "git을 찾지 못했습니다."
}

$currentCommit = (git rev-parse HEAD).Trim()
$tagCommit = (git rev-list -n 1 $Tag).Trim()

if ($currentCommit -ne $tagCommit) {
  throw "태그 $Tag 가 현재 커밋을 가리키지 않습니다. current=$currentCommit tag=$tagCommit"
}

$archiveDir = Join-Path $env:TEMP "ds5-release-source-check-$Tag"
$archiveZip = Join-Path $env:TEMP "ds5-release-source-check-$Tag.zip"
Remove-Item -LiteralPath $archiveDir -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $archiveZip -Force -ErrorAction SilentlyContinue

$archiveUrl = "https://github.com/$Repository/archive/refs/tags/$Tag.zip"
Invoke-WebRequest -Uri $archiveUrl -OutFile $archiveZip
Expand-Archive -LiteralPath $archiveZip -DestinationPath $archiveDir -Force

$root = Get-ChildItem -LiteralPath $archiveDir -Directory | Select-Object -First 1
if (-not $root) {
  throw "릴리즈 소스 아카이브를 풀지 못했습니다."
}

$requiredPaths = @(
  "config-manager/package.json",
  "config-manager/src/App.svelte",
  "config-manager/src/styles.css",
  "config-manager/src/lib/i18n.ts",
  "config-manager/src/lib/components/SettingsModal.svelte",
  "config-manager/src-tauri/tauri.conf.json",
  "config-manager/src-tauri/src/bridge.rs",
  "src/main.cpp",
  "CMakeLists.txt"
)

$missing = @()
foreach ($relativePath in $requiredPaths) {
  $path = Join-Path $root.FullName $relativePath
  if (-not (Test-Path -LiteralPath $path)) {
    $missing += $relativePath
  }
}

if ($missing.Count -gt 0) {
  throw "릴리즈 소스 아카이브에 필수 파일이 없습니다: $($missing -join ', ')"
}

Write-Host "Release source verified: $Repository@$Tag ($currentCommit)"
