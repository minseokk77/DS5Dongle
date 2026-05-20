//
// Created by awalol on 2026/5/15.
//

#ifndef DS5_BRIDGE_STATE_MGR_H
#define DS5_BRIDGE_STATE_MGR_H

void state_init();
void state_set(uint8_t *data, const uint8_t size);
void state_update(const uint8_t *data, const uint8_t size);
void state_set_mic_muted(bool muted);
bool state_get_mic_muted();

#endif //DS5_BRIDGE_STATE_MGR_H
