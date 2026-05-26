//
// Created by awalol on 2026/3/4.
//

#ifndef DS5_BRIDGE_USB_H
#define DS5_BRIDGE_USB_H

#include <cstdint>

enum UsbPresentationMode : uint8_t {
    USB_PRESENTATION_CONFIG_ONLY = 0,
    USB_PRESENTATION_DUALSENSE_COMPOSITE = 1,
};

extern uint8_t mute[2]; // 0: SPEAKER(0x02) 1: MIC(0x05)
extern float volume[2]; // 0: SPEAKER(0x02) 1: MIC(0x05)

UsbPresentationMode usb_get_presentation_mode();
void usb_set_presentation_mode(UsbPresentationMode mode, bool reconnect);

#endif //DS5_BRIDGE_USB_H
