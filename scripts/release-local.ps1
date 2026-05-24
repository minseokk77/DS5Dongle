param(
  [string]$Tag = "",

  [string]$Repository = "minseokk77/DS5Dongle",

  [string]$FirmwareUf2Path = "",

  [switch]$NoBuild
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

function Get-NextReleaseTag {
  param([string]$Repository)

  $tags = @()
  try {
    $tags += @(gh release list --repo $Repository --limit 100 --json tagName --jq '.[].tagName')
  } catch {
    Write-Host "GitHub 릴리즈 목록을 읽지 못해 로컬 태그만 사용합니다."
  }
  $tags += @(git tag --list "v0.0.*.*")
  $latest = $tags |
    Where-Object { $_ -match '^v0\.0\.(\d+)\.(\d+)$' } |
    Sort-Object {
      if ($_ -match '^v0\.0\.(\d+)\.(\d+)$') {
        ([int]$Matches[1] * 100000) + [int]$Matches[2]
      } else {
        0
      }
    } -Descending |
    Select-Object -First 1

  if (-not $latest -or $latest -notmatch '^v0\.0\.(\d+)\.(\d+)$') {
    return "v0.0.1.0"
  }

  return "v0.0.$($Matches[1]).$([int]$Matches[2] + 1)"
}

function Get-BundleVersionFromRelease {
  param([string]$ReleaseVersion)

  if ($ReleaseVersion -notmatch '^0\.0\.(\d+)\.(\d+)$') {
    throw "릴리즈 버전은 0.0.x.x 형식이어야 합니다."
  }

  return "0.0.$($Matches[1])$($Matches[2])"
}

function Set-TextFileUtf8 {
  param(
    [string]$Path,
    [string]$Content
  )

  $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
  [System.IO.File]::WriteAllText($Path, $Content, $utf8NoBom)
}

function Update-ProjectVersionFiles {
  param(
    [string]$ReleaseVersion,
    [string]$BundleVersion
  )

  $appPath = Join-Path $configManager "src\App.svelte"
  $packagePath = Join-Path $configManager "package.json"
  $cargoPath = Join-Path $configManager "src-tauri\Cargo.toml"
  $cargoLockPath = Join-Path $configManager "src-tauri\Cargo.lock"
  $tauriConfigPath = Join-Path $configManager "src-tauri\tauri.conf.json"

  Set-TextFileUtf8 $appPath ((Get-Content -Raw -Encoding UTF8 $appPath) -replace "const appVersion = '[^']+';", "const appVersion = '$ReleaseVersion';")
  Set-TextFileUtf8 $packagePath ((Get-Content -Raw -Encoding UTF8 $packagePath) -replace '"version":\s*"[^"]+"', """version"": ""$BundleVersion""")
  Set-TextFileUtf8 $cargoPath ((Get-Content -Raw -Encoding UTF8 $cargoPath) -replace 'version\s*=\s*"[^"]+"', "version = ""$BundleVersion""")
  Set-TextFileUtf8 $cargoLockPath ((Get-Content -Raw -Encoding UTF8 $cargoLockPath) -replace '(name = "ds5-bridge-config-tauri"\s+version = )"[^"]+"', "`${1}""$BundleVersion""")
  Set-TextFileUtf8 $tauriConfigPath ((Get-Content -Raw -Encoding UTF8 $tauriConfigPath) -replace '"version":\s*"[^"]+"', """version"": ""$BundleVersion""")
}

if (-not $Tag) {
  $preBumpDirty = git status --porcelain
  if ($preBumpDirty) {
    throw "자동 버전 증가 전 작업 트리가 깨끗해야 합니다. 먼저 커밋하거나 되돌리세요."
  }

  $Tag = Get-NextReleaseTag -Repository $Repository
  $releaseVersionForBump = $Tag.TrimStart("v")
  $bundleVersionForBump = Get-BundleVersionFromRelease -ReleaseVersion $releaseVersionForBump
  Update-ProjectVersionFiles -ReleaseVersion $releaseVersionForBump -BundleVersion $bundleVersionForBump
  Push-Location $repoRoot
  try {
    $versionFiles = @(
      (Join-Path $configManager "src\App.svelte"),
      (Join-Path $configManager "package.json"),
      (Join-Path $configManager "src-tauri\Cargo.toml"),
      (Join-Path $configManager "src-tauri\Cargo.lock"),
      (Join-Path $configManager "src-tauri\tauri.conf.json")
    )
    git add @versionFiles
    git commit -m "Bump version to $releaseVersionForBump"
  }
  finally {
    Pop-Location
  }
}

if ($Tag -notmatch '^v0\.0\.\d+\.\d+$') {
  throw "릴리즈 태그는 v0.0.x.x 형식이어야 합니다. 예: v0.0.1.4"
}

$releaseVersion = $Tag.TrimStart("v")
$assetVersionCandidates = @($releaseVersion)
if ($releaseVersion -match '^0\.0\.(\d+)\.(\d+)$') {
  # Tauri/Cargo/MSI는 3자리 semver만 안정적으로 처리하므로,
  # v0.0.1.4 같은 릴리즈는 앱 번들 0.0.14와 매칭해서 업로드합니다.
  $compactPatch = "$($Matches[1])$($Matches[2])"
  $assetVersionCandidates += "0.0.$compactPatch"
  $assetVersionCandidates += "0.$($Matches[1]).$($Matches[2])"
}

function New-ReleaseNotesFile {
  param(
    [string]$Tag,
    [string]$Repository,
    [string]$AssetStage,
    [string[]]$Assets
  )

  $notesPath = Join-Path $AssetStage "release-notes-$($Tag.TrimStart('v')).md"
  $previousTag = @(git tag --list "v0.0.*.*" --sort=-v:refname | Where-Object { $_ -ne $Tag } | Select-Object -First 1)
  $range = if ($previousTag) { "$previousTag..HEAD" } else { "HEAD" }
  $changes = @(git log $range --pretty=format:"- %s")
  if ($changes.Count -eq 0) {
    $changes = @("- 소스와 빌드 에셋을 현재 태그 기준으로 정리했습니다.")
  }
  $assetLines = @($Assets | ForEach-Object { "- $(Split-Path -Leaf $_)" })
  $notes = @"
로컬 빌드 릴리즈입니다.

- GitHub Actions는 사용하지 않았습니다.
- 앱 설치 파일과 debug UF2를 로컬에서 빌드해 업로드했습니다.
- 앱과 펌웨어 업데이트 경로는 $Repository 릴리즈를 기준으로 동작합니다.
- 공식 DS5Dongle 기반: v0.6.0-hotfix

## 변경 사항
$($changes -join "`n")

## 에셋
$($assetLines -join "`n")

업데이트 전 USB를 분리하지 말고, 앱의 펌웨어 업데이트 진행 창이 완료될 때까지 기다려 주세요.
"@

  $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
  [System.IO.File]::WriteAllText($notesPath, $notes, $utf8NoBom)
  return $notesPath
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

  if (-not $NoBuild) {
    Push-Location $configManager
    try {
      pnpm.cmd install --frozen-lockfile
    }
    finally {
      Pop-Location
    }

    & (Join-Path $repoRoot "scripts\pre-release-check.ps1") -Repository $Repository

    Push-Location $configManager
    try {
      pnpm.cmd tauri build
    }
    finally {
      Pop-Location
    }
  }
  else {
    Write-Host "NoBuild 옵션이 켜져 있어 앱 빌드와 사전 체크를 건너뜁니다."
  }

  $bundleRoot = Join-Path $configManager "src-tauri\target\release\bundle"
  $assetStage = Join-Path $repoRoot "target\release-assets\$releaseVersion"
  New-Item -ItemType Directory -Force -Path $assetStage | Out-Null
  $assets = @()
  $bundleAssets = Get-ChildItem -LiteralPath $bundleRoot -Recurse -File |
    Where-Object {
      $assetName = $_.Name
      $_.Extension -in ".exe", ".msi" -and
      [bool]($assetVersionCandidates | Where-Object { $assetName -like "*$_*" } | Select-Object -First 1)
    }

  foreach ($asset in $bundleAssets) {
    $stagedName = $asset.Name
    if ($stagedName -notlike "*$releaseVersion*") {
      $stagedName = $stagedName -replace '\d+\.\d+\.\d+(?:\.\d+)?', $releaseVersion
    }
    $stagedName = $stagedName -replace '\s+', '.'
    $stagedPath = Join-Path $assetStage $stagedName
    Copy-Item -LiteralPath $asset.FullName -Destination $stagedPath -Force
    $assets += $stagedPath
  }

  if ($FirmwareUf2Path) {
    $resolvedUf2 = (Resolve-Path -LiteralPath $FirmwareUf2Path).Path
    $stagedUf2 = Join-Path $assetStage "ds5-bridge-debug-$Tag.uf2"
    Copy-Item -LiteralPath $resolvedUf2 -Destination $stagedUf2 -Force
    $assets += $stagedUf2
  }

  if ($assets.Count -eq 0) {
    throw "업로드할 릴리즈 에셋을 찾지 못했습니다."
  }

  $releaseNotesPath = New-ReleaseNotesFile -Tag $Tag -Repository $Repository -AssetStage $assetStage -Assets $assets

  $releaseTags = @(gh release list --repo $Repository --limit 100 --json tagName --jq '.[].tagName')
  $releaseExists = $releaseTags -contains $Tag

  if (-not $releaseExists) {
    gh release create $Tag --repo $Repository --title $Tag --notes-file $releaseNotesPath
  }
  else {
    gh release edit $Tag --repo $Repository --title $Tag --notes-file $releaseNotesPath
  }

  gh release upload $Tag --repo $Repository @assets --clobber
  & (Join-Path $repoRoot "scripts\verify-release-source.ps1") -Tag $Tag -Repository $Repository
}
finally {
  Pop-Location
}
