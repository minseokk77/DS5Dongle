//
// Created by awalol on 2026/5/4.
//

#include "cmd.h"

#include <algorithm>
#include <cstdio>
#include <cstring>

#include "bt.h"
#include "config.h"
#include "device/usbd.h"
#include "pico/time.h"
#include "pico/bootrom.h"
#include "audio.h"
#include "wake.h"

// spk_active (main.cpp) + audio_mic_active() (audio.cpp) are surfaced in the
// 0xf9 feature report so the config UI can display the real gated mic/speaker
// state, reflecting the disable_mic / disable_speaker settings.
extern bool spk_active;
extern uint8_t interrupt_in_data[63];

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
        memset(buffer, 0, reqlen);
        const auto len = std::min(strlen(PICO_PROGRAM_VERSION_STRING), static_cast<size_t>(reqlen - 1));
        memcpy(buffer, PICO_PROGRAM_VERSION_STRING, len);
        return len;
    }
    if (report_id == 0xfa) {
        printf("[HID] Receive 0xfa getting capabilities\n");
        static_assert(sizeof(Config_body) <= 255);
        uint8_t capabilities[24] = {
            'D', '5', 'C', 'P',
            1, // CAPABILITY_PROTOCOL_VERSION
            get_config().config_version,
            static_cast<uint8_t>(sizeof(Config_body)),
            0,
            0x3f, // FEATURE_FLAGS >> 0
            0x00, // FEATURE_FLAGS >> 8
            0x00, // FEATURE_FLAGS >> 16
            0x00, // FEATURE_FLAGS >> 24
        };
        capabilities[12] = bt_is_connected() ? 1 : 0;
        const char* build_channel = "release";
#if ENABLE_VERBOSE
        build_channel = "debug";
#endif
        const auto channel_len = std::min(strlen(build_channel), sizeof(capabilities) - 14);
        capabilities[13] = static_cast<uint8_t>(channel_len);
        memcpy(capabilities + 14, build_channel, channel_len);
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
        // byte 1: real audio gating state, for the config UI to display.
        //   bit7 = valid marker (firmware without this byte leaves it 0)
        //   bit0 = controller mic actually streaming (host opened it AND !disable_mic)
        //   bit1 = controller speaker actually driven (host opened it AND !disable_speaker)
        if (reqlen >= 2) {
            uint8_t flags = 0x80;
            if (audio_mic_active() && !get_config().disable_mic) flags |= 0x01;
            if (spk_active && !get_config().disable_speaker) flags |= 0x02;
            buffer[1] = flags;
            return 2;
        }
#if ENABLE_VERBOSE
        printf("[HID] 0xf9 RSSI=%d raw=0x%02X\n", rssi, buffer[0]);
#endif
        return 1;
    }
    if (report_id == 0xf5) {
        if (!bt_is_connected()) return 0;

        const uint8_t b   = interrupt_in_data[52];
        const uint8_t pct = b & 0x0F;
        const uint8_t st  = (b >> 4) & 0x0F;
        uint8_t level = (pct <= 11) ? (pct * 10) : pct;
        if (level > 100) level = 100;

        if (reqlen == 0) return 0;
        buffer[0] = level;
        if (reqlen >= 2) buffer[1] = st;
        if (reqlen >= 3) buffer[2] = b;

#if ENABLE_VERBOSE
        printf("[HID] 0xf5 battery=%u state=0x%02X raw=0x%02X\n", buffer[0], (reqlen >= 2 ? buffer[1] : 0), (reqlen >= 3 ? buffer[2] : 0));
#endif
        return std::min(reqlen, (uint16_t)3);
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
    // 0x03 reconnect tinyusb device;
    if (buffer[0] == 0x01) {
#if ENABLE_VERBOSE
        printf("[CMD] Enter config set func\n");
#endif
        set_config(buffer + 1, bufsize - 1);
    }
    if (buffer[0] == 0x02) {
        printf("[CMD] Enter config save func\n");
        config_save();
    }
    if (buffer[0] == 0x03) {
        printf("[CMD] Enter tud reconnect func\n");
        wake_note_usb_reconnect();   // this disconnect is intentional, not a host sleep
        tud_disconnect();
        sleep_ms(150);
        tud_connect();
    }
    if (buffer[0] == 0x04) {
        printf("[CMD] Enter bootloader mode\n");
        reset_usb_boot(0, 0);
    }
}
