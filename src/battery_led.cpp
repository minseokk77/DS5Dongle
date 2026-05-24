//
// Low-battery LED indicator. See battery_led.h.
//

#include "battery_led.h"

#include <cstdint>

#include "bt.h"
#include "config.h"
#include "pico/cyw43_arch.h"
#include "pico/time.h"
#include "state_mgr.h"
#include "utils.h"

extern uint8_t interrupt_in_data[63];
extern int reportSeqCounter;

namespace {

constexpr uint64_t REPORT_STALE_US = 2'000'000;  // assume disconnected if no report for 2 s
constexpr uint64_t BLINK_PERIOD_US =   500'000;  // 1 Hz, 50% duty
constexpr uint8_t  THRESHOLD_LEVEL = 1;          // PowerPercent <= 1 (i.e. <= 10%)
constexpr uint8_t  POWER_STATE_DISCHARGING = 0x0;

uint64_t last_report_us = 0;
uint64_t last_toggle_us = 0;
bool     blinking       = false;
bool     led_state      = false;

void send_dualsense_state_report(void) {
    if (!bt_is_controller_connected()) {
        return;
    }

    uint8_t output_data[78]{};
    output_data[0] = 0x31;
    output_data[1] = reportSeqCounter << 4;
    if (++reportSeqCounter == 256) {
        reportSeqCounter = 0;
    }
    output_data[2] = 0x10;
    state_set(output_data + 3, sizeof(SetStateData));
    bt_write(output_data, sizeof(output_data));
}

}  // namespace

void battery_led_init(void) {
    last_report_us = 0;
    last_toggle_us = 0;
    blinking = false;
    led_state = false;
}

void battery_led_note_report(void) {
    last_report_us = time_us_64();
}

bool battery_led_get_status(uint8_t *level_percent, uint8_t *power_state, uint8_t *raw_value) {
    const uint64_t now = time_us_64();
    if (last_report_us == 0 || (now - last_report_us) >= REPORT_STALE_US) {
        return false;
    }

    const uint8_t raw = interrupt_in_data[52];
    const uint8_t pct = raw & 0x0F;
    const uint8_t st = (raw >> 4) & 0x0F;

    if (level_percent) {
        *level_percent = pct >= 10 ? 100 : pct * 10;
    }
    if (power_state) {
        *power_state = st;
    }
    if (raw_value) {
        *raw_value = raw;
    }
    return true;
}

void battery_led_on_disconnect(void) {
    blinking = false;
    led_state = false;
    last_report_us = 0;
    last_toggle_us = 0;
    state_set_low_battery_warning(false, false);
    cyw43_arch_gpio_put(CYW43_WL_GPIO_LED_PIN, false);
}

void battery_led_tick(void) {
    const uint64_t now = time_us_64();
    if (last_report_us == 0 || (now - last_report_us) >= REPORT_STALE_US) {
        // No fresh data: force off if a blink was active, then wait for a fresh report.
        if (blinking) {
            blinking = false;
            led_state = false;
            state_set_low_battery_warning(false, false);
            cyw43_arch_gpio_put(CYW43_WL_GPIO_LED_PIN, false);
        }
        return;
    }

    const uint8_t b   = interrupt_in_data[52];
    const uint8_t pct = b & 0x0F;
    const uint8_t st  = (b >> 4) & 0x0F;
    const bool low    = (st == POWER_STATE_DISCHARGING) && (pct <= THRESHOLD_LEVEL);

    if (low) {
        if (!blinking) {
            blinking = true;
            led_state = true;
            last_toggle_us = now;
            state_set_low_battery_warning(true, true);
            send_dualsense_state_report();
            cyw43_arch_gpio_put(CYW43_WL_GPIO_LED_PIN, !get_config().disable_pico_led);
            return;
        }
        if ((now - last_toggle_us) >= BLINK_PERIOD_US) {
            led_state = !led_state;
            last_toggle_us = now;
            state_set_low_battery_warning(true, led_state);
            send_dualsense_state_report();
        }
    } else if (blinking) {
        blinking = false;
        state_set_low_battery_warning(false, false);
        send_dualsense_state_report();
        // Battery recovered or now charging — restore Pico steady-state LED per preference.
        cyw43_arch_gpio_put(CYW43_WL_GPIO_LED_PIN, !get_config().disable_pico_led);
    }
}
