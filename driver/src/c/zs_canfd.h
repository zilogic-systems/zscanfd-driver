#ifndef ZS_CANFD_H_
#define ZS_CANFD_H_

#ifdef __cplusplus
extern "C" {
#endif  // __cplusplus
	#include <stdint.h>

	typedef void* zs_canfdhnd;

	struct CanFrame {
		uint8_t channel;
		uint32_t id;
		uint8_t dlc;
		uint8_t data[64];
		uint8_t ext;
		uint8_t fd;
		uint8_t brs;
		uint8_t esi;
		uint8_t loopback;
		uint8_t rtr;
		uint8_t err;
		uint32_t timestamp_us;
		const char* dev_ser_num_ref;
	};

	typedef void(__cdecl* ZsCanFdRxCallback)(const struct CanFrame* f);

	__declspec(dllimport) zs_canfdhnd zs_canfd_init();
	__declspec(dllimport) int32_t zs_canfd_deinit(zs_canfdhnd hnd);

	__declspec(dllimport) int32_t zs_canfd_open(zs_canfdhnd hnd);
	__declspec(dllimport) int32_t zs_canfd_open_with_device_serial_number(zs_canfdhnd hnd, const char* dev_ser_num_ref);
	__declspec(dllimport) int32_t zs_canfd_close(zs_canfdhnd hnd);

	__declspec(dllimport) int32_t zs_canfd_set_rx_callback(zs_canfdhnd hnd, ZsCanFdRxCallback callback);

	__declspec(dllimport) int32_t zs_canfd_start(zs_canfdhnd hnd, uint8_t channel);
	__declspec(dllimport) int32_t zs_canfd_stop(zs_canfdhnd hnd, uint8_t channel);

	__declspec(dllimport) int32_t zs_canfd_transmit(zs_canfdhnd hnd, const struct CanFrame f);

	__declspec(dllimport) int32_t zs_canfd_set_bitrate(zs_canfdhnd hnd, uint8_t channel, uint32_t bitrate);
	__declspec(dllimport) int32_t zs_canfd_set_data_bitrate(zs_canfdhnd hnd, uint8_t channel, uint32_t bitrate);
	__declspec(dllimport) int32_t zs_canfd_set_enabled(zs_canfdhnd hnd, uint8_t channel, uint8_t enabled);
	__declspec(dllimport) int32_t zs_canfd_set_monitor(zs_canfdhnd hnd, uint8_t channel, uint8_t enabled);
	__declspec(dllimport) int32_t zs_canfd_set_hw_loopback(zs_canfdhnd hnd, uint8_t channel, uint8_t enabled);

	__declspec(dllimport) int32_t zs_canfd_get_channel_count(zs_canfdhnd hnd);

	__declspec(dllimport) int32_t zs_canfd_set_fd(zs_canfdhnd hnd, uint8_t channel, uint8_t enabled);

	__declspec(dllimport) int32_t zs_canfd_get_available_devices_count();
	__declspec(dllimport) const char* zs_canfd_get_available_devices_serial_number();
	__declspec(dllimport) int32_t free_string(const char* ptr);
#ifdef __cplusplus
}
#endif  // __cplusplus

#endif
