# Pico2W DualSense 5 Bridge

[한국어](#한국어) | [中文](#中文) | [English](#english)

## 한국어

> Raspberry Pi Pico2W를 DualSense 컨트롤러용 무선 어댑터로 사용하는 펌웨어입니다.

### 개요

이 프로젝트는 Raspberry Pi Pico2W를 DualSense 컨트롤러용 Bluetooth 브리지로 동작하게 합니다. 무선 연결과 HD 햅틱을 지원하며, 이 포크는 네이티브 `DS5 Bridge Config` 데스크톱 앱과 함께 쓰도록 맞춰져 있습니다.

이 포크의 GitHub 릴리즈에는 앱이 설치에 사용하는 `debug` UF2만 올립니다. 일반 UF2는 앱 업데이트 흐름에서 사용하지 않기 때문에 배포하지 않습니다.

### 기능

- Pico2W를 통한 DualSense 무선 연결
- HD 햅틱 지원
- `DS5 Bridge Config` 데스크톱 앱으로 설정 변경
- GitHub Releases의 `debug` UF2를 이용한 앱 내 펌웨어 업데이트

### 펌웨어 설치

권장 방식:

1. `DS5 Bridge Config` 데스크톱 앱을 실행합니다.
2. DS5 Bridge 장치를 USB로 연결합니다.
3. 앱의 펌웨어 업데이트 기능을 실행합니다.
4. 앱이 GitHub Releases에서 최신 `debug` UF2를 다운로드하고, Pico 부트로더에 진입한 뒤 UF2를 자동으로 복사합니다.

수동 방식:

1. Pico2W의 `BOOTSEL` 버튼을 누른 상태로 USB를 연결합니다.
2. Windows에 `RP2350` 또는 `RPI-RP2` 드라이브가 나타나는지 확인합니다.
3. `ds5-bridge-debug-*.uf2` 파일을 해당 드라이브에 복사합니다.

### 컨트롤러 페어링

1. DualSense 컨트롤러를 Bluetooth 페어링 모드로 전환합니다.
2. Pico2W가 컨트롤러를 감지하고 연결될 때까지 기다립니다.
3. 컨트롤러가 연결된 뒤에 호스트 시스템에 장치가 표시됩니다.

컨트롤러가 페어링 모드일 때 Pico를 다시 연결해야 할 수 있습니다.

### 설정

Pico 설정은 `DS5 Bridge Config` 데스크톱 앱에서 변경합니다.

이 포크의 햅틱 강도 범위는 `0.1`부터 `2.0`까지입니다. 음수 햅틱 강도 값은 앱과 펌웨어 양쪽에서 거부됩니다.

업스트림 웹 설정 페이지는 참고용으로 남아 있습니다.

- Release: https://ds5.awalol.eu.org
- Development: https://ds5-dev.awalol.eu.org

### 참고

- 컨트롤러가 Pico에 연결된 뒤에만 시스템에 장치가 표시됩니다.
- 일부 설정은 USB 재연결 뒤에 적용됩니다.
- 저전력 LED 경고는 컨트롤러 배터리가 10% 이하이고 충전 중이 아닐 때 Pico LED를 1 Hz로 깜빡이게 합니다.
- 빌드 시 `-DENABLE_BATT_LED=OFF`를 사용하면 저전력 LED 경고를 끌 수 있습니다. 기본값은 ON입니다.
- Pico W 빌드는 햅틱만 지원하고 스피커는 지원하지 않습니다. `-DPICO_W_BUILD=ON`으로 빌드할 수 있습니다.
- USB Wake 기능은 실험적 기능이며, 필요하면 업스트림 `feat/usb-wake` 브랜치를 참고하세요.

### 알려진 문제

- 오디오가 약간 끊길 수 있습니다.
- 인코딩 성능 때문에 Pico2W 오버클럭이 필요합니다.

현재 오버클럭 설정:

- 전압: 1.2V
- 주파수: 320 MHz

부팅에 실패하면 전압을 조금 올리거나 CPU 주파수를 낮춰야 할 수 있습니다.

### 빌드

표준 Pico SDK 툴체인으로 빌드합니다. 업스트림과 마찬가지로 Pico SDK 안의 TinyUSB 버전이 최신이어야 합니다.

## 中文

> 将 Raspberry Pi Pico2W 作为 DualSense 控制器的无线适配器使用。

### 概览

本项目让 Raspberry Pi Pico2W 作为 DualSense 控制器的 Bluetooth 桥接器工作，支持无线连接和 HD 触觉反馈。本 fork 针对原生 `DS5 Bridge Config` 桌面应用进行了适配。

此 fork 的 GitHub Releases 只发布应用更新流程使用的 `debug` UF2。普通 UF2 不会被应用安装，因此不再发布。

### 功能

- 通过 Pico2W 连接 DualSense
- 支持 HD 触觉反馈
- 通过 `DS5 Bridge Config` 桌面应用修改设置
- 在应用内从 GitHub Releases 下载并安装 `debug` UF2

### 安装固件

推荐方式：

1. 打开 `DS5 Bridge Config` 桌面应用。
2. 通过 USB 连接 DS5 Bridge 设备。
3. 在应用中执行固件更新。
4. 应用会从 GitHub Releases 下载最新的 `debug` UF2，进入 Pico bootloader，并自动复制 UF2。

手动方式：

1. 按住 Pico2W 的 `BOOTSEL` 按钮并连接 USB。
2. 确认系统出现 `RP2350` 或 `RPI-RP2` 磁盘。
3. 将 `ds5-bridge-debug-*.uf2` 复制到该磁盘。

### 控制器配对

1. 将 DualSense 控制器切换到 Bluetooth 配对模式。
2. 等待 Pico2W 检测并连接控制器。
3. 控制器连接后，主机系统才会显示该设备。

控制器处于配对模式时，可能需要重新插拔 Pico。

### 配置

Pico 设置通过 `DS5 Bridge Config` 桌面应用修改。

此 fork 的触觉强度范围为 `0.1` 到 `2.0`。应用和固件都会拒绝负数触觉强度。

上游网页配置页面仍可作为参考：

- Release: https://ds5.awalol.eu.org
- Development: https://ds5-dev.awalol.eu.org

### 说明

- 控制器连接到 Pico 后，系统才会显示设备。
- 部分设置需要 USB 重新连接后生效。
- 当控制器电量低于或等于 10% 且未充电时，Pico LED 会以 1 Hz 闪烁作为低电量提醒。
- 构建时可使用 `-DENABLE_BATT_LED=OFF` 关闭低电量 LED 提醒。默认开启。
- Pico W 构建只支持触觉反馈，不支持扬声器。可使用 `-DPICO_W_BUILD=ON` 构建。
- USB Wake 功能仍为实验性功能，如需使用请参考上游 `feat/usb-wake` 分支。

### 已知问题

- 音频可能会有轻微卡顿。
- 由于编码性能需求，Pico2W 需要超频。

当前超频设置：

- 电压：1.2V
- 频率：320 MHz

如果设备无法启动，请适当提高电压或降低 CPU 频率。

### 构建

使用标准 Pico SDK 工具链构建。与上游相同，Pico SDK 中的 TinyUSB 需要更新到较新版本。

## English

> Turn a Raspberry Pi Pico2W into a wireless adapter for the DualSense controller.

### Overview

This project enables the Raspberry Pi Pico2W to work as a Bluetooth bridge for the DualSense controller, with wireless connectivity and HD haptics support. This fork is tuned for the native `DS5 Bridge Config` desktop app.

This fork publishes only the `debug` UF2 used by the app's updater. The standard UF2 is not used by the app update flow, so it is not published.

### Features

- DualSense wireless connectivity through Pico2W
- HD haptics support
- Configuration through the native `DS5 Bridge Config` desktop app
- In-app firmware updates using the `debug` UF2 from GitHub Releases

### Flashing Firmware

Recommended path:

1. Open the `DS5 Bridge Config` desktop app.
2. Connect the DS5 Bridge device over USB.
3. Run the firmware update action in the app.
4. The app downloads the latest `debug` UF2 from GitHub Releases, enters the Pico bootloader, and copies the UF2 automatically.

Manual path:

1. Hold the `BOOTSEL` button on the Pico2W while connecting USB.
2. Confirm that an `RP2350` or `RPI-RP2` drive appears.
3. Copy `ds5-bridge-debug-*.uf2` to that drive.

### Pairing the Controller

1. Put the DualSense controller into Bluetooth pairing mode.
2. Wait for the Pico2W to detect and connect to it.
3. The device appears on the host system after the controller is connected.

You may need to replug the Pico while the controller is in pairing mode.

### Configuration

Pico settings are changed through the `DS5 Bridge Config` desktop app.

This fork keeps the haptics gain range at `0.1` to `2.0`. Negative haptics gain values are rejected by both the app and firmware.

The original upstream web config remains available for reference:

- Release: https://ds5.awalol.eu.org
- Development: https://ds5-dev.awalol.eu.org

### Notes

- The Pico device is visible to the system only after the controller is connected.
- Some settings require a USB reconnect before they take effect.
- The low-battery LED warning blinks the Pico LED at 1 Hz when the controller reports 10% battery or lower and is not charging.
- To disable the low-battery LED warning at build time, use `-DENABLE_BATT_LED=OFF`. Default is ON.
- Pico W builds support haptics only, not speaker output. Build with `-DPICO_W_BUILD=ON`.
- USB Wake is experimental. Refer to the upstream `feat/usb-wake` branch if you need it.

### Known Issues

- Audio may experience slight stuttering.
- Pico2W overclocking is required for proper encoding performance.

Current overclock settings:

- Voltage: 1.2V
- Frequency: 320 MHz

If your device fails to boot, increase voltage slightly or reduce CPU frequency.

### Build

Build with the standard Pico SDK toolchain. As with upstream, TinyUSB in the Pico SDK should be updated to a recent version.

## References

- [rafaelvaloto/Pico_W-Dualsense](https://github.com/rafaelvaloto/Pico_W-Dualsense) - Project inspiration
- [egormanga/SAxense](https://github.com/egormanga/SAxense) - Bluetooth haptics proof of concept
- [Sony DualSense data report structure](https://controllers.fandom.com/wiki/Sony_DualSense)
- [Paliverse/DualSenseX](https://github.com/Paliverse/DualSenseX) - Speaker report packet
