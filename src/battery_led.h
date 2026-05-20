//
// Low-battery LED indicator for the Pico onboard LED.
// Reads PowerPercent / PowerState from interrupt_in_data[52]
// (DualSense BT 0x31 report, see USBGetStateData in utils.h).
//

#pragma once

#include <cstdint>

void battery_led_init(void);

// Call once per main-loop iteration. Drives the LED blink while the
// battery is low and the controller is connected; otherwise no-op.
void battery_led_tick(void);

// Call from the BT input-report callback whenever a fresh 0x31 report
// has been copied into interrupt_in_data. Used to detect disconnection
// via stale-report timeout.
void battery_led_note_report(void);

// 최신 DualSense 입력 리포트에서 실제 배터리 상태를 읽는다.
// 반환값이 false이면 최근 리포트가 없어서 값이 유효하지 않은 상태다.
bool battery_led_get_status(uint8_t *level_percent, uint8_t *power_state, uint8_t *raw_value);
