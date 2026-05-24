//
// Created by awalol on 2026/5/4.
//

#include "cmd.h"

#include <algorithm>
#include <cstdio>
#include <cstring>

#include "bt.h"
#include "battery_led.h"
#include "config.h"
#include "device/usbd.h"
#include "pico/bootrom.h"
#include "pico/time.h"

constexpr uint8_t CAPABILITY_PROTOCOL_VERSION = 1;
constexpr uint32_t FEATURE_BATTERY = 1u << 0;
constexpr uint32_t FEATURE_RSSI = 1u << 1;
constexpr uint32_t FEATURE_VIBRATION_TEST = 1u << 2;
constexpr uint32_t FEATURE_ADAPTIVE_TRIGGER = 1u << 3;
constexpr uint32_t FEATURE_BOOTLOADER_COMMAND = 1u << 4;
constexpr uint32_t FEATURE_STICK_CALIBRATION = 1u << 5;
constexpr uint32_t FEATURE_DIRECTIONAL_STICK_CALIBRATION = 1u << 6;
constexpr uint32_t FEATURE_FLAGS =
    FEATURE_BATTERY |
    FEATURE_RSSI |
    FEATURE_VIBRATION_TEST |
    FEATURE_ADAPTIVE_TRIGGER |
    FEATURE_BOOTLOADER_COMMAND |
    FEATURE_STICK_CALIBRATION |
    FEATURE_DIRECTIONAL_STICK_CALIBRATION;

bool is_pico_cmd(uint8_t report_id) {
    if (report_id == 0xf6 ||
        report_id == 0xf7 ||
        report_id == 0xf5 ||
        report_id == 0xf8 ||
        report_id == 0xf9 ||
        report_id == 0xfa
    ) {
        return true;
    }
    return false;
}

uint16_t pico_cmd_get(uint8_t report_id, uint8_t *buffer, uint16_t reqlen) {
    if (report_id == 0xf7) {
        printf("[HID] Receive 0xf7 getting config\n");
        if (sizeof(Config_body) > reqlen) {
            printf("[Config] Warning: Config_body overflow\n");
        }
        const auto len = std::min(sizeof(Config_body),static_cast<size_t>(reqlen));
        memcpy(buffer,&get_config(),len);
        return len;
    }
    if (report_id == 0xf8) {
        printf("[HID] Receive 0xf8 getting firmware version\n");
        const auto len = std::min(strlen(PICO_PROGRAM_VERSION_STRING), static_cast<size_t>(reqlen));
        memcpy(buffer, PICO_PROGRAM_VERSION_STRING, len);
        return len;
    }
    if (report_id == 0xfa) {
        printf("[HID] Receive 0xfa getting capabilities\n");
        static_assert(sizeof(Config_body) <= 255);
        uint8_t capabilities[12] = {
            'D', '5', 'C', 'P',
            CAPABILITY_PROTOCOL_VERSION,
            get_config().config_version,
            static_cast<uint8_t>(sizeof(Config_body)),
            0,
            static_cast<uint8_t>((FEATURE_FLAGS >> 0) & 0xff),
            static_cast<uint8_t>((FEATURE_FLAGS >> 8) & 0xff),
            static_cast<uint8_t>((FEATURE_FLAGS >> 16) & 0xff),
            static_cast<uint8_t>((FEATURE_FLAGS >> 24) & 0xff),
        };
        const auto len = std::min(sizeof(capabilities), static_cast<size_t>(reqlen));
        memcpy(buffer, capabilities, len);
        return len;
    }
    if (report_id == 0xf9) {
        // [-128,0]
        int8_t rssi = 0;
        bt_get_signal_strength(&rssi);
        if (reqlen == 0) {
            return 0;
        }
        buffer[0] = rssi;
#if ENABLE_VERBOSE
        printf("[HID] 0xf9 RSSI=%d raw=0x%02X\n", rssi, buffer[0]);
#endif
        return 1;
    }
    if (report_id == 0xf5) {
        uint8_t level = 0;
        uint8_t state = 0;
        uint8_t raw = 0;
        if (reqlen < 3 || !battery_led_get_status(&level, &state, &raw)) {
            return 0;
        }
        buffer[0] = level;
        buffer[1] = state;
        buffer[2] = raw;
#if ENABLE_VERBOSE
        printf("[HID] 0xf5 battery=%u state=0x%02X raw=0x%02X\n", level, state, raw);
#endif
        return 3;
    }
    return 0;
}

void pico_cmd_set(uint8_t report_id, uint8_t const *buffer, uint16_t bufsize) {
    (void) report_id;
    if (bufsize == 0) {
        return;
    }

    // 0x01 update config in variable
    // 0x02 write config to flash
    // 0x03 reconnect tinyusb device
    // 0x04 reboot into USB bootloader
    if (buffer[0] == 0x01) {
        printf("[CMD] Enter config set func\n");
        set_config(buffer + 1, bufsize - 1);
    }
    if (buffer[0] == 0x02) {
        printf("[CMD] Enter config save func\n");
        config_save();
    }
    if (buffer[0] == 0x03) {
        printf("[CMD] Enter tud reconnect func\n");
        tud_disconnect();
        sleep_ms(150);
        tud_connect();
    }
    if (buffer[0] == 0x04) {
        printf("[CMD] Enter USB bootloader\n");
        tud_disconnect();
        sleep_ms(150);
        reset_usb_boot(0, 0);
    }
}
