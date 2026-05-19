# Pico2W DualSense 5 Bridge

[中文](./README.CN.md)

> Turn a Raspberry Pi Pico2W into a wireless adapter for the DualSense (DS5) controller.

## Overview

This project enables the Raspberry Pi Pico2W to function as a Bluetooth bridge for the DualSense controller, allowing wireless connectivity with enhanced haptics support.

This fork is tuned for the native DS5 Bridge Config desktop app. Release firmware assets are published as debug UF2 files only, because the app's firmware updater intentionally installs the debug UF2 from GitHub Releases.

## Features

- 🎮 Full DualSense connectivity via Pico2W
- 🔊 Supports HD haptics (advanced vibration feedback)
- 📡 Wireless Bluetooth bridging
- 🛠 Native DS5 Bridge Config app support with GitHub Release based firmware updates

## Getting Started

### Flashing Firmware

Recommended path for this fork:

1. Open the DS5 Bridge Config desktop app
2. Connect the DS5 Bridge device over USB
3. Use the firmware update action in the app
4. The app downloads the latest `debug` UF2 from this repository's GitHub Releases, enters the Pico bootloader, and copies the UF2 automatically

Manual path:

1. Hold the BOOTSEL button on the Pico2W
2. Connect the Pico2W to your computer via USB
3. The device will mount as a USB storage device
4. Drag and drop the `ds5-bridge-debug-*.uf2` firmware file onto the device

### Pairing the Controller

1. Put the DualSense controller into Bluetooth pairing mode
2. Wait for the Pico2W to detect and connect
3. Once connected, the device will appear on the host system

***You may need to replug the Pico when the controller is in pairing mode.***

## Configuration

You can modify the Pico settings via the DS5 Bridge Config desktop app.

This fork keeps the haptics gain range at `0.1` to `2.0`. Negative haptics gain values are rejected by both the app and firmware.

The original upstream web config is still available:

- For release: https://ds5.awalol.eu.org
- For development: https://ds5-dev.awalol.eu.org

## Notes

The Pico device will only be visible to the system after the controller is connected

Some behaviors depend on reconnection cycles to take effect

### Low-battery LED indicator

When the connected DualSense reports its battery at or below 10% (and it is not charging), the Pico onboard LED switches from solid-on to a 1 Hz blink so you can see the warning at a glance. The LED returns to solid-on as soon as the controller is plugged in or its reported level rises again. The blink also fires when `disable_pico_led` is set — the warning is treated as critical and overrides the LED-off preference; the LED returns to its disabled (off) state once the battery recovers or the controller starts charging.

To opt out at build time, configure with `-DENABLE_BATT_LED=OFF`. Default is ON.

### Pico W Version

Pico W only has haptics support, no speaker. You can enable Pico W firmware compilation with `-DPICO_W_BUILD=ON`, or download precompiled firmware from GitHub Actions.

### USB Wake Feature

This feature is experimental. If you need this functionality, please check out the feat/usb-wake branch to compile it, or use the precompiled firmware from GitHub Actions under that branch. The `ds5-bridge-wake.uf2` is the firmware with this feature enabled.

It is recommended to read #60 and #61 before using this feature.

### Community Fork
https://github.com/MarcelineVPQ/DS5Dongle-OLED-Edition
https://github.com/zurce/DS5Dongle-OLED

## Known Issues

- ⚠️ Audio may experience slight stuttering
- ⚠️ Overclocking is required for proper performance

## Performance / Overclocking

Due to encoding requirements, the Pico2W must be overclocked:

Current settings:

- Voltage: 1.2V
- Frequency: 320 MHz

If your device fails to boot:

- Increase voltage slightly or Reduce CPU frequency

## Build Instructions

To build the project from source:

1. ***Update TinyUSB in the Pico SDK to the latest version***
2. Compile using standard Pico SDK toolchain

## Roadmap
- Please check out [DS5Dongle plan](https://github.com/users/awalol/projects/5)

## Community
- Join the Discord server: [Discord Server](https://discord.gg/hM4ntchGCa)
- If you have a bug, please open an issue instead.

## References

- [rafaelvaloto/Pico_W-Dualsense](https://github.com/rafaelvaloto/Pico_W-Dualsense) — Project inspiration
- [egormanga/SAxense](https://github.com/egormanga/SAxense) — Bluetooth Haptics POC
- [https://controllers.fandom.com/wiki/Sony_DualSense](https://controllers.fandom.com/wiki/Sony_DualSense) - DualSense data report structure documentation
- [Paliverse/DualSenseX](https://github.com/Paliverse/DualSenseX) — Speaker report packet
