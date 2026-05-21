//
// Created by awalol on 2026/5/4.
//

#ifndef DS5_BRIDGE_CONFIG_H
#define DS5_BRIDGE_CONFIG_H

#include <cstdint>

struct __attribute__((packed)) Config_body {
    uint8_t config_version; // Config Version
    float haptics_gain; // [0.1,2.0]
    float speaker_volume; // [-100,0]
    uint8_t inactive_time; // [5,60] min
    uint8_t disable_inactive_disconnect; // bool: 0 disable,1 enable
    uint8_t disable_pico_led; // bool
    uint8_t polling_rate_mode; // 0: 250Hz, 1: 500Hz, 2: real-time
    uint8_t audio_buffer_length; // [16,128]
    uint8_t controller_mode; // 0: DS5, 1: DSE, 2: Auto
    uint8_t stick_calibration_enabled; // bool
    int8_t left_stick_center_x; // signed offset from 128
    int8_t left_stick_center_y; // signed offset from 128
    uint8_t left_stick_deadzone; // percent * 10, [0,80]
    int8_t right_stick_center_x; // signed offset from 128
    int8_t right_stick_center_y; // signed offset from 128
    uint8_t right_stick_deadzone; // percent * 10, [0,80]
    int8_t left_stick_min_x; // signed reach from center, [-127,0]
    int8_t left_stick_max_x; // signed reach from center, [0,127]
    int8_t left_stick_min_y; // signed reach from center, [-127,0]
    int8_t left_stick_max_y; // signed reach from center, [0,127]
    int8_t right_stick_min_x; // signed reach from center, [-127,0]
    int8_t right_stick_max_x; // signed reach from center, [0,127]
    int8_t right_stick_min_y; // signed reach from center, [-127,0]
    int8_t right_stick_max_y; // signed reach from center, [0,127]
};

struct __attribute__((packed)) Config {
    uint32_t magic;
    uint16_t version;
    uint32_t crc32; // Config_body crc32, only calc and verify when save
    uint16_t size;  // Config_body size
    Config_body body;
};

void config_default();
void config_load();
bool config_save();
const Config_body& get_config();
void set_config(const uint8_t *new_config, const uint16_t len);
void config_valid();
void set_config(const Config_body &new_config);
void apply_stick_calibration(uint8_t *report);
extern bool is_dse;

#endif //DS5_BRIDGE_CONFIG_H
