use crate::bridge;
use crate::settings::settings;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::Path,
    ptr,
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

#[derive(Debug, Serialize)]
pub struct BootloaderStatus {
    pub available: bool,
    pub drive: Option<String>,
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

pub async fn check_debug_firmware_update() -> Result<FirmwareUpdateInfo, UpdateError> {
    let update_settings = &settings().firmware_update;
    let releases_api = format!(
        "https://api.github.com/repos/{}/{}/releases",
        update_settings.github_owner, update_settings.github_repo
    );
    let keyword = update_settings.debug_asset_keyword.to_ascii_lowercase();
    
    let response = github_client()
        .get(releases_api)
        .send()
        .await?
        .error_for_status()?;
        
    let releases = response.json::<Vec<GitHubRelease>>().await?;

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

pub async fn flash_latest_debug_firmware(
    device_id: Option<String>,
) -> Result<FirmwareFlashResult, UpdateError> {
    let _autoplay_guard = AutoPlayGuard::disable_for_update();
    let update = check_debug_firmware_update().await?;
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
    
    let response = github_client()
        .get(&update.download_url)
        .send()
        .await?
        .error_for_status()?;
        
    let bytes = response.bytes().await?;

    let target = Path::new(&drive).join(&update.asset_name);
    fs::write(&target, bytes)?;

    Ok(FirmwareFlashResult {
        version: update.version,
        asset_name: update.asset_name,
        drive,
    })
}

struct AutoPlayGuard {
    previous: Option<u32>,
    changed: bool,
}

impl AutoPlayGuard {
    fn disable_for_update() -> Self {
        let previous = read_autoplay_disabled();
        let changed = write_autoplay_disabled(Some(1)).is_ok();
        Self { previous, changed }
    }
}

impl Drop for AutoPlayGuard {
    fn drop(&mut self) {
        if self.changed {
            let _ = write_autoplay_disabled(self.previous);
        }
    }
}

#[cfg(windows)]
fn read_autoplay_disabled() -> Option<u32> {
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE,
        REG_DWORD,
    };

    let key_path = wide_null("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\AutoplayHandlers");
    let value_name = wide_null("DisableAutoplay");
    let mut key: HKEY = ptr::null_mut();
    let open_result = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            key_path.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut key,
        )
    };
    if open_result != 0 {
        return None;
    }

    let mut value_type = 0_u32;
    let mut value = 0_u32;
    let mut value_size = std::mem::size_of::<u32>() as u32;
    let query_result = unsafe {
        RegQueryValueExW(
            key,
            value_name.as_ptr(),
            ptr::null_mut(),
            &mut value_type,
            &mut value as *mut u32 as *mut u8,
            &mut value_size,
        )
    };
    unsafe {
        RegCloseKey(key);
    }

    (query_result == 0 && value_type == REG_DWORD).then_some(value)
}

#[cfg(not(windows))]
fn read_autoplay_disabled() -> Option<u32> {
    None
}

#[cfg(windows)]
fn write_autoplay_disabled(value: Option<u32>) -> Result<(), ()> {
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegCreateKeyW, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY,
        HKEY_CURRENT_USER, KEY_SET_VALUE, REG_DWORD,
    };

    let key_path = wide_null("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\AutoplayHandlers");
    let value_name = wide_null("DisableAutoplay");
    let mut key: HKEY = ptr::null_mut();

    let open_result = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            key_path.as_ptr(),
            0,
            KEY_SET_VALUE,
            &mut key,
        )
    };

    if open_result != 0 {
        let create_result = unsafe {
            RegCreateKeyW(
                HKEY_CURRENT_USER,
                key_path.as_ptr(),
                &mut key,
            )
        };
        if create_result != 0 {
            return Err(());
        }
    }

    let result = if let Some(mut next_value) = value {
        unsafe {
            RegSetValueExW(
                key,
                value_name.as_ptr(),
                0,
                REG_DWORD,
                &mut next_value as *mut u32 as *const u8,
                std::mem::size_of::<u32>() as u32,
            )
        }
    } else {
        unsafe { RegDeleteValueW(key, value_name.as_ptr()) }
    };

    unsafe {
        RegCloseKey(key);
    }

    (result == 0).then_some(()).ok_or(())
}

#[cfg(not(windows))]
fn write_autoplay_disabled(_value: Option<u32>) -> Result<(), ()> {
    Ok(())
}

#[cfg(windows)]
fn wide_null(value: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(value).encode_wide().chain([0]).collect()
}

pub fn bootloader_status() -> Result<BootloaderStatus, UpdateError> {
    let drive = find_bootloader_drive_optional()?;
    Ok(BootloaderStatus {
        available: drive.is_some(),
        drive,
    })
}

pub async fn recovery_flash_latest_debug_firmware() -> Result<FirmwareFlashResult, UpdateError> {
    flash_latest_debug_firmware(None).await
}

fn github_client() -> reqwest::Client {
    reqwest::Client::builder()
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

/// Windows API를 사용하여 주어진 Volume Label에 해당하는 드라이브 경로를 찾습니다.
fn find_drive_by_label(label: &str) -> Result<Option<String>, UpdateError> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use windows_sys::Win32::Storage::FileSystem::GetVolumeInformationW;

    // A:\ 부터 Z:\ 까지 논리 드라이브 순회
    for letter in b'A'..=b'Z' {
        let drive_letter = letter as char;
        let root_path = format!("{}:\\\0", drive_letter);
        let root_path_w: Vec<u16> = OsStr::new(&root_path).encode_wide().collect();
        let mut volume_name = [0u16; 261];

        let success = unsafe {
            GetVolumeInformationW(
                root_path_w.as_ptr(),
                volume_name.as_mut_ptr(),
                volume_name.len() as u32,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
            )
        };

        if success != 0 {
            let len = volume_name.iter().position(|&x| x == 0).unwrap_or(volume_name.len());
            let name = String::from_utf16_lossy(&volume_name[..len]);
            if name.trim().to_ascii_lowercase() == label.to_ascii_lowercase() {
                return Ok(Some(format!("{}:\\", drive_letter)));
            }
        }
    }

    Ok(None)
}