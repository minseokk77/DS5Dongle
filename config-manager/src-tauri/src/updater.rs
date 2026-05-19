use crate::bridge;
use crate::settings::settings;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{Duration, Instant},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("GitHub 릴리즈 정보를 가져오지 못했습니다. {0}")]
    Request(#[from] reqwest::Error),
    #[error("공식 릴리즈에서 debug UF2 파일을 찾지 못했습니다.")]
    NoDebugAsset,
    #[error("RP2350 또는 RPI-RP2 부트로더 드라이브를 찾지 못했습니다. BOOTSEL을 누른 상태로 Pico를 다시 연결하세요.")]
    BootDriveNotFound,
    #[error("펌웨어 파일을 저장하지 못했습니다. {0}")]
    Io(#[from] std::io::Error),
    #[error("Windows 볼륨 정보를 읽지 못했습니다. {0}")]
    Volume(String),
    #[error("부트로더 진입 명령을 보내지 못했습니다. {0}")]
    Bridge(#[from] bridge::BridgeError),
}

impl serde::Serialize for UpdateError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Serialize)]
pub struct FirmwareUpdateInfo {
    pub version: String,
    pub asset_name: String,
    pub download_url: String,
}

#[derive(Debug, Serialize)]
pub struct FirmwareFlashResult {
    pub version: String,
    pub asset_name: String,
    pub drive: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    draft: bool,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

pub fn check_debug_firmware_update() -> Result<FirmwareUpdateInfo, UpdateError> {
    let update_settings = &settings().firmware_update;
    let releases_api = format!(
        "https://api.github.com/repos/{}/{}/releases",
        update_settings.github_owner, update_settings.github_repo
    );
    let keyword = update_settings.debug_asset_keyword.to_ascii_lowercase();
    let releases = github_client()
        .get(releases_api)
        .send()?
        .error_for_status()?
        .json::<Vec<GitHubRelease>>()?;

    releases
        .into_iter()
        .filter(|release| !release.draft)
        .find_map(|release| {
            release
                .assets
                .into_iter()
                .find(|asset| {
                    let name = asset.name.to_ascii_lowercase();
                    name.ends_with(".uf2") && name.contains(&keyword)
                })
                .map(|asset| FirmwareUpdateInfo {
                    version: release.tag_name,
                    asset_name: asset.name,
                    download_url: asset.browser_download_url,
                })
        })
        .ok_or(UpdateError::NoDebugAsset)
}

pub fn flash_latest_debug_firmware(
    device_id: Option<String>,
) -> Result<FirmwareFlashResult, UpdateError> {
    let update = check_debug_firmware_update()?;
    let drive = match find_bootloader_drive_optional()? {
        Some(drive) => drive,
        None => {
            if let Some(device_id) = device_id.as_deref().filter(|value| !value.is_empty()) {
                bridge::enter_bootloader(device_id)?;
                wait_for_bootloader_drive(Duration::from_secs(18))?
            } else {
                return Err(UpdateError::BootDriveNotFound);
            }
        }
    };
    let bytes = github_client()
        .get(&update.download_url)
        .send()?
        .error_for_status()?
        .bytes()?;

    let target = Path::new(&drive).join(&update.asset_name);
    fs::write(&target, bytes)?;

    Ok(FirmwareFlashResult {
        version: update.version,
        asset_name: update.asset_name,
        drive,
    })
}

fn github_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(45))
        .user_agent("DS5-Bridge-Config/0.1")
        .build()
        .expect("HTTP 클라이언트 생성에 실패했습니다.")
}

fn find_bootloader_drive_optional() -> Result<Option<String>, UpdateError> {
    for label in ["RP2350", "RPI-RP2"] {
        if let Some(drive) = find_drive_by_label(label)? {
            return Ok(Some(drive));
        }
    }

    Ok(None)
}

fn wait_for_bootloader_drive(timeout: Duration) -> Result<String, UpdateError> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(drive) = find_bootloader_drive_optional()? {
            return Ok(drive);
        }
        if Instant::now() >= deadline {
            return Err(UpdateError::BootDriveNotFound);
        }
        thread::sleep(Duration::from_millis(350));
    }
}

fn find_drive_by_label(label: &str) -> Result<Option<String>, UpdateError> {
    let script = format!(
        "(Get-Volume -FileSystemLabel '{}' -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty DriveLetter)",
        label
    );
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()?;

    if !output.status.success() {
        return Err(UpdateError::Volume(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    let letter = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if letter.is_empty() {
        return Ok(None);
    }

    let drive = PathBuf::from(format!("{letter}:\\"));
    Ok(Some(drive.to_string_lossy().to_string()))
}
