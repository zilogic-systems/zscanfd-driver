// txrx.c
//
// A simple C application to demonstrate sending and receiving classic CAN frames
// using the zscanfd.dll API on Windows.


#include <stdio.h>
#include <windows.h>  // Required for Sleep()
#include "../../src/c/zscanfd.h"

// --- Receive Callback Function ---
// This function is called by the zscanfd library whenever a CAN frame is received.
// It must match the function pointer signature from zscanfd_set_rx_callback.
void __cdecl rx_callback(const struct CanFrame* frame) {
    if (frame == NULL) {
        return;
    }

    printf("<- RX | ID: 0x%03X | DLC: %d | Data: ", frame->id, frame->dlc);
    for (int i = 0; i < frame->dlc; i++) {
        printf("%02X ", frame->data[i]);
    }
    printf("\n");
}

int main() {
    // 1. Initialize the zs_canfd library and get a handle
    zs_canfdhnd hnd = zs_canfd_init();
    if (hnd == NULL) {
        printf("Error: Failed to initialize zs_canfd library.\n");
        return -1;
    }
    printf("Library initialized successfully.\n");

    // 2. Open the device connection
    const u_char dev_serial_num[] = "1234"; // Replace with valid serial number
    if (zs_canfd_open(hnd, dev_serial_num) != 0) {
        printf("Error: Failed to open device.\n");
        zs_canfd_deinit(hnd);
        return -6;
    }
    printf("Device opened.\n");

    // Check how many channels are available
    int32_t channel_count = zs_canfd_get_channel_count(hnd);
    if (channel_count <= 0) {
        printf("Error: No CAN channels found.\n");
        zs_canfd_deinit(hnd);
        return -2;
    }
    printf("Found %d channel(s).\n", channel_count);


    // 3. Configure Channel 0 for classic CAN
    uint8_t channel = 0;
    uint32_t bitrate = 100000;  // 100 kbit/s
    printf("Configuring Channel %d with bitrate %u bps...\n", channel, bitrate);

    if (zs_canfd_set_bitrate(hnd, channel, bitrate) != 0) {
        printf("Error: Failed to set bitrate.\n");
        zs_canfd_deinit(hnd);
        return -3;
    }

    if (zs_canfd_set_enabled(hnd, channel, 1) != 0) {
        printf("Error: Failed to enable channel.\n");
        zs_canfd_deinit(hnd);
        return -4;
    }

    // 4. Set the receive callback function
    if (zs_canfd_set_rx_callback(hnd, rx_callback) != 0) {
        printf("Error: Failed to set RX callback.\n");
        zs_canfd_deinit(hnd);
        return -5;
    }
    printf("RX callback registered.\n");

    // 5. Start communication on the CAN bus
    if (zs_canfd_start(hnd, channel) != 0) {
        printf("Error: Failed to start communication.\n");
        zs_canfd_close(hnd);
        zs_canfd_deinit(hnd);
        return -1;
    }
    printf("CAN bus communication started.\n\n");

    // 6. Prepare and transmit a classic CAN frame
    struct CanFrame tx_frame = {
        .channel = channel,
        .id = 0x123,
        .dlc = 8,
        .data = {
            [0] = 0xDE,
            [1] = 0xAD,
            [2] = 0xBE,
            [3] = 0xEF,
            [4] = 0xFE,
            [5] = 0xED,
            [6] = 0xFA,
            [7] = 0xCE,
        },
        .ext = 0,  // 0 for 11-bit standard ID, 1 for 29-bit extended ID
        .fd = 0,  // 0 for classic CAN, 1 for CAN-FD
        .brs = 0,  // Bit Rate Switch (CAN-FD only)
        .esi = 0,  // Error State Indicator (CAN-FD only)
        .loopback = 0,  // 0 to disable software loopback
        .rtr = 0,  // 0 for data frame, 1 for remote frame
        .err = 0,  // Not an error frame
    };

    printf("-> TX | ID: 0x%03X | DLC: %d | Data: ", tx_frame.id, tx_frame.dlc);
    for (int i = 0; i < tx_frame.dlc; i++) {
        printf("%02X ", tx_frame.data[i]);
    }
    printf("\n");
    if (zs_canfd_transmit(hnd, tx_frame) != 0) {
        printf("Error: Failed to transmit frame.\n");
    }

    // 7. Wait and listen for incoming frames
    printf("\nListening for CAN frames for 10 seconds...\n");
    Sleep(10000);  // Wait for 10 seconds (10,000 milliseconds)

    // 8. Stop and clean up
    printf("\nStopping communication...\n");
    zs_canfd_stop(hnd, channel);
    printf("Closing device...\n");
    zs_canfd_close(hnd);
    printf("Deinitializing library...\n");
    zs_canfd_deinit(hnd);

    printf("--- Program Finished ---\n");
    return 0;
}
