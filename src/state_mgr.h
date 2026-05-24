//
// Created by awalol on 2026/5/15.
//

#ifndef DS5_BRIDGE_STATE_MGR_H
#define DS5_BRIDGE_STATE_MGR_H

#include <cstdint>

void state_init();
void state_set(uint8_t *data, const uint8_t size);
void state_update(const uint8_t *data, const uint8_t size);
void state_set_mic_muted(bool muted);
bool state_get_mic_muted();
void state_set_low_battery_warning(bool active, bool light_on);

#endif //DS5_BRIDGE_STATE_MGR_H
