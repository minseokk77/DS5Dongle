//
// Created by awalol on 2026/5/4.
//

#include "config.h"

#include <cmath>
#include <cstring>

#include "bt.h"
#include "state_mgr.h"
#include "utils.h"
#include "hardware/flash.h"
#include "hardware/sync.h"
#include "pico/cyw43_arch.h"
#include "pico/flash.h"

constexpr uint32_t CONFIG_MAGIC = 0x66ccff00;
constexpr uint16_t CONFIG_VERSION = 5;
constexpr uint32_t CONFIG_FLASH_OFFSET = PICO_FLASH_SIZE_BYTES - FLASH_SECTOR_SIZE;
static Config config{};
bool is_dse = false;

// 编译期保护
// 判断Config结构体是否能放进flash 256bytes
static_assert(sizeof(Config) <= FLASH_PAGE_SIZE);
// 配置区起始地址必须按 flash sector 对齐。
static_assert(CONFIG_FLASH_OFFSET % FLASH_SECTOR_SIZE == 0);

void default_stick_calibration(Config_body *body) {
    body->stick_calibration_enabled = 0;
    body->left_stick_center_x = 0;
    body->left_stick_center_y = 0;
    body->left_stick_deadzone = 10;
    body->right_stick_center_x = 0;
    body->right_stick_center_y = 0;
    body->right_stick_deadzone = 10;
}

void default_directional_stick_calibration(Config_body *body) {
    body->left_stick_min_x = -127;
    body->left_stick_max_x = 127;
    body->left_stick_min_y = -127;
    body->left_stick_max_y = 127;
    body->right_stick_min_x = -127;
    body->right_stick_max_x = 127;
    body->right_stick_min_y = -127;
    body->right_stick_max_y = 127;
}

uint32_t calc_config_crc(const Config &con) {
    return crc32(reinterpret_cast<const uint8_t *>(&con.body), sizeof(Config_body));
}

const Config *flash_config() {
    return reinterpret_cast<const Config *>(XIP_BASE + CONFIG_FLASH_OFFSET);
}

void config_valid() {
    bool reset_all_stick_calibration = false;
    bool reset_directional_calibration = false;
    const uint16_t previous_body_size = config.size;
    const uint8_t previous_body_version = config.body.config_version;
    if (config.magic != CONFIG_MAGIC) {
        config.magic = CONFIG_MAGIC;
        reset_all_stick_calibration = true;
        printf("[Config] Config Magic Header is invalid\n");
    }
    if (config.size != sizeof(Config_body)) {
        config.size = sizeof(Config_body);
        reset_directional_calibration = previous_body_size < sizeof(Config_body);
        printf("[Config] Config Body size is invalid\n");
    }
    auto body = &config.body;
    if (body->config_version != CONFIG_VERSION) {
        memset(body, 0, sizeof(Config_body));
        body->config_version = CONFIG_VERSION;
        reset_directional_calibration = previous_body_version < CONFIG_VERSION;
        printf("[Config] Warning: Config may breaking change. Reset to default\n");
    }
    if (std::isnan(body->haptics_gain) || body->haptics_gain < 1.0f || body->haptics_gain > 2.0f) {
        body->haptics_gain = 1.0f;
        printf("[Config] Haptics Gain value is invalid\n");
    }
    if (body->speaker_volume < 0 || body->speaker_volume > 127) {
        body->speaker_volume = 100;
        printf("[Config] Speaker Volume is invalid\n");
    }
    if (body->headset_volume < 0 || body->headset_volume > 127) {
        body->headset_volume = 100;
        printf("[Config] Headset Volume is invalid\n");
    }
    if (body->speaker_gain < 0 || body->speaker_gain > 7) {
        body->speaker_gain = 2;
        printf("[Config] speaker_gain is invalid\n");
    }
    if (body->inactive_time < 0 || body->inactive_time > 60) {
        body->inactive_time = 30;
        printf("[Config] Inactive time is invalid\n");
    }
    if (body->disable_pico_led > 1) {
        body->disable_pico_led = 0;
        printf("[Config] disable_pico_led is invalid\n");
    }
    if (body->polling_rate_mode > 2) {
        body->polling_rate_mode = 1;
        printf("[Config] polling_rate_mode is invalid\n");
    }
    if (body->audio_buffer_length < 16 || body->audio_buffer_length > 128) {
        body->audio_buffer_length = 64;
        printf("[Config] haptics_buffer_length is invalid\n");
    }
    if (body->controller_mode > 2) {
        body->controller_mode = 2;
        printf("[Config] controller_mode is invalid\n");
    }
    if (body->enable_usb_sn > 1) {
        body->enable_usb_sn = 0;
        printf("[Config] Warning: enable_usb_sn is invalid\n");
    }
    if (body->ps_shortcut_enabled > 1) {
        body->ps_shortcut_enabled = 0;
        printf("[Config] ps_shortcut_enabled is invalid\n");
    }
    if (body->disable_mic > 1) {
        body->disable_mic = 0;
        printf("[Config] disable_mic is invalid\n");
    }
    if (body->disable_speaker > 1) {
        body->disable_speaker = 0;
        printf("[Config] disable_speaker is invalid\n");
    }
    if (body->enable_wake > 1) {
        body->enable_wake = 0;
        printf("[Config] enable_wake is invalid\n");
    }
    if (body->trigger_reduce > 10) {
        body->trigger_reduce = 0;
        printf("[Config] trigger_reduce is invalid\n");
    }

    if (reset_all_stick_calibration) {
        default_stick_calibration(body);
        default_directional_stick_calibration(body);
    } else if (reset_directional_calibration) {
        default_directional_stick_calibration(body);
    }
    if (body->stick_calibration_enabled > 1) {
        body->stick_calibration_enabled = 0;
        printf("[Config] stick_calibration_enabled is invalid\n");
    }
    if (body->left_stick_deadzone > 80) {
        body->left_stick_deadzone = 10;
        printf("[Config] left_stick_deadzone is invalid\n");
    }
    if (body->right_stick_deadzone > 80) {
        body->right_stick_deadzone = 10;
        printf("[Config] right_stick_deadzone is invalid\n");
    }
    if (body->left_stick_min_x >= 0 || body->left_stick_min_x < -127) body->left_stick_min_x = -127;
    if (body->left_stick_max_x <= 0 || body->left_stick_max_x > 127) body->left_stick_max_x = 127;
    if (body->left_stick_min_y >= 0 || body->left_stick_min_y < -127) body->left_stick_min_y = -127;
    if (body->left_stick_max_y <= 0 || body->left_stick_max_y > 127) body->left_stick_max_y = 127;
    if (body->right_stick_min_x >= 0 || body->right_stick_min_x < -127) body->right_stick_min_x = -127;
    if (body->right_stick_max_x <= 0 || body->right_stick_max_x > 127) body->right_stick_max_x = 127;
    if (body->right_stick_min_y >= 0 || body->right_stick_min_y < -127) body->right_stick_min_y = -127;
    if (body->right_stick_max_y <= 0 || body->right_stick_max_y > 127) body->right_stick_max_y = 127;
}

void config_load() {
    memcpy(&config, flash_config(), sizeof(Config));

    config_valid();
}

// Runs with core1 parked (flash_safe_execute) and core0 interrupts disabled, so
// neither core touches XIP flash while the sector is erased/programmed. Without
// the core1 park this races the audio core and corrupts audio (buzzing).
static void config_save_flash_op(void *param) {
    const uint8_t *page = static_cast<const uint8_t *>(param);
    const uint32_t interrupts = save_and_disable_interrupts();
    flash_range_erase(CONFIG_FLASH_OFFSET, FLASH_SECTOR_SIZE);
    flash_range_program(CONFIG_FLASH_OFFSET, page, FLASH_PAGE_SIZE);
    restore_interrupts(interrupts);
}

bool config_save() {
    config.crc32 = calc_config_crc(config);
    alignas(4) uint8_t page[FLASH_PAGE_SIZE];
    memset(page, 0xff, sizeof(page));
    memcpy(page, &config, sizeof(Config));

    const int rc = flash_safe_execute(config_save_flash_op, page, 1000);
    if (rc != PICO_OK) {
        printf("[Config] config_save flash_safe_execute failed: %d\n", rc);
        return false;
    }

    Config verify{};
    memcpy(&verify, flash_config(), sizeof(verify));
    const auto verify_crc32 = calc_config_crc(verify);
    if (verify_crc32 == config.crc32) {
        printf("[Config] Config write flash verify success\n");
        return true;
    }
    printf("[Config] Config write flash verify failed\n");
    return false;
}

Config_body& get_config() {
    return config.body;
}

void set_config(const uint8_t *new_config, const uint16_t len) {
    const auto copy_len = len < sizeof(Config_body) ? len : sizeof(Config_body);
    memcpy(&config.body, new_config, copy_len);
    config_valid();
    if (config.body.disable_pico_led) {
        cyw43_arch_gpio_put(CYW43_WL_GPIO_LED_PIN, false);
    } else {
        if (bt_is_connected()) {
            cyw43_arch_gpio_put(CYW43_WL_GPIO_LED_PIN, true);
        }
    }
    set_volume(config.body.speaker_volume,config.body.headset_volume);
    if (config.body.speaker_gain != 0) {
        set_gain(config.body.speaker_gain);
    }
    if (config.body.trigger_reduce != 0) {
        set_trigger_reduce(config.body.trigger_reduce);
    }
}

void set_config(const Config_body &new_config) {
    config.body = new_config;
    config_valid();
}

static uint8_t apply_axis_calibration(uint8_t raw, int8_t center_offset, uint8_t deadzone_tenths, int8_t min_reach, int8_t max_reach) {
    const int center = 128 + static_cast<int>(center_offset);
    int value = static_cast<int>(raw) - center;
    const int deadzone = static_cast<int>(deadzone_tenths) * 127 / 1000;

    if (std::abs(value) <= deadzone) {
        return 128;
    }

    const int reach = value > 0 ? static_cast<int>(max_reach) : -static_cast<int>(min_reach);
    const int range = reach - deadzone;
    if (range <= 0) {
        return 128;
    }

    if (value > 0) {
        value = (value - deadzone) * 127 / range;
    } else {
        value = (value + deadzone) * 127 / range;
    }

    const int corrected = 128 + value;
    if (corrected < 0) return 0;
    if (corrected > 255) return 255;
    return static_cast<uint8_t>(corrected);
}

void apply_stick_calibration(uint8_t *report) {
    const auto &body = get_config();
    if (!body.stick_calibration_enabled || report == nullptr) {
        return;
    }

    report[0] = apply_axis_calibration(report[0], body.left_stick_center_x, body.left_stick_deadzone, body.left_stick_min_x, body.left_stick_max_x);
    report[1] = apply_axis_calibration(report[1], body.left_stick_center_y, body.left_stick_deadzone, body.left_stick_min_y, body.left_stick_max_y);
    report[2] = apply_axis_calibration(report[2], body.right_stick_center_x, body.right_stick_deadzone, body.right_stick_min_x, body.right_stick_max_x);
    report[3] = apply_axis_calibration(report[3], body.right_stick_center_y, body.right_stick_deadzone, body.right_stick_min_y, body.right_stick_max_y);
}
