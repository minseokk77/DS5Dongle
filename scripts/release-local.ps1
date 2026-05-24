param(
  [Parameter(Mandatory = $true)]
  [string]$Tag,

  [string]$Repository = "minseokk77/DS5Dongle",

  [string]$FirmwareUf2Path = ""
)

$ErrorActionPreference = "Stop"

function Require-Command {
  param([string]$Name)
  if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
    throw "$Name 명령을 찾지 못했습니다."
  }
}

Require-Command git
Require-Command gh
Require-Command pnpm.cmd
Require-Command cargo

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$configManager = Join-Path $repoRoot "config-manager"

if (-not (Test-Path -LiteralPath $configManager)) {
  throw "config-manager 폴더를 찾지 못했습니다."
}

Push-Location $repoRoot
try {
  $dirty = git status --porcelain
  if ($dirty) {
    throw "작업 트리가 깨끗하지 않습니다. 먼저 커밋하거나 되돌린 뒤 릴리즈하세요."
  }

  $currentCommit = (git rev-parse HEAD).Trim()
  git tag -f $Tag $currentCommit | Out-Null
  git push origin main
  git push --force origin $Tag

  Push-Location $configManager
  try {
    pnpm.cmd install --frozen-lockfile
    pnpm.cmd build
    cargo check --manifest-path ".\src-tauri\Cargo.toml"
    pnpm.cmd tauri build
  }
  finally {
    Pop-Location
  }

  $bundleRoot = Join-Path $configManager "src-tauri\target\release\bundle"
  $assets = @()
  $assets += Get-ChildItem -LiteralPath $bundleRoot -Recurse -File |
    Where-Object { $_.Extension -in ".exe", ".msi" } |
    Select-Object -ExpandProperty FullName

  if ($FirmwareUf2Path) {
    $resolvedUf2 = (Resolve-Path -LiteralPath $FirmwareUf2Path).Path
    $assets += $resolvedUf2
  }

  if ($assets.Count -eq 0) {
    throw "업로드할 릴리즈 에셋을 찾지 못했습니다."
  }

  $releaseTags = @(gh release list --repo $Repository --limit 100 --json tagName --jq '.[].tagName')
  $releaseExists = $releaseTags -contains $Tag

  if (-not $releaseExists) {
    gh release create $Tag --repo $Repository --title $Tag --notes "로컬 빌드 릴리즈입니다. GitHub Actions는 사용하지 않습니다."
  }

  gh release upload $Tag --repo $Repository @assets --clobber
  & (Join-Path $repoRoot "scripts\verify-release-source.ps1") -Tag $Tag -Repository $Repository
}
finally {
  Pop-Location
}
