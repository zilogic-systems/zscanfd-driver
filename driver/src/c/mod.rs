//! Implementation of C/C++ bindings.
//!
//! All functions are unsafe since they dereference a context pointer
//! provided from C.
//!
//! TODO: put a simple example here.
//!

#![allow(clippy::missing_safety_doc)]

use crate::{Frame, Interface};
use std::ffi::{CStr, c_char};
use std::ffi::CString;
use std::ptr::null_mut;

use crate::{libusb_helper};
use libusb1_sys::libusb_context;
use libusb1_sys::libusb_init;
use libusb1_sys::constants::LIBUSB_SUCCESS;
use std::mem;

// zscanfd USB VID / PID
const USB_VID: u16 = 0x1d50;
const USB_PID: u16 = 0x606f;

/// A CAN frame in a C representation
#[repr(C)]
pub struct CFrame {
    channel: u8,
    id: u32,
    dlc: u8,
    data: [u8; 64],
    // these types are boolean flags, but C FFI hates bools
    // use u8s instead: 1 = true, 0 = false
    ext: u8,
    fd: u8,
    brs: u8,
    esi: u8,
    loopback: u8,
    rtr: u8,
    err: u8,
    timestamp_us: u32,
    dev_ser_num_ref: *mut c_char,
}

impl CFrame {
    fn from_frame(f: Frame, dev_ser_num: CString) -> CFrame {
        CFrame {
            channel: f.channel,
            id: f.can_id,
            dlc: f.can_dlc,
            data: f.data_as_array(),
            ext: if f.ext { 1 } else { 0 },
            fd: if f.fd { 1 } else { 0 },
            brs: if f.brs { 1 } else { 0 },
            esi: if f.esi { 1 } else { 0 },
            loopback: if f.loopback { 1 } else { 0 },
            rtr: if f.rtr { 1 } else { 0 },
            err: if f.err { 1 } else { 0 },
            timestamp_us: f.hw_timestamp,
            dev_ser_num_ref : dev_ser_num.into_raw(),
        }
    }
}

/// Interface state. A pointer to this struct is provided when initializing the
/// library. All other functions require a pointer to this struct as the first
/// argument.
#[repr(C)]
pub struct CInterface {
    i: Option<Interface>,
    c_rx_cb: Option<extern "C" fn(*const CFrame)>,
    dev_ser_num: String,
}

/// Create a new zs_canfd interface, returning a pointer to the interface.
/// This pointer must be provided as the first argument to all other calls in
/// this library.
///
/// If this function fails, it returns a null pointer (0).
#[no_mangle]
pub extern "C" fn zs_canfd_init() -> *mut CInterface {
    Box::into_raw(Box::new(CInterface {
        i: None,
        c_rx_cb: None,
        dev_ser_num: String::new(),
    }))
}

/// Clean up a zs_canfd interface.
/// After calling, the pointer is no longer valid.
#[no_mangle]
pub unsafe extern "C" fn zs_canfd_deinit(ptr: *mut CInterface) -> i32 {
    let _ = Box::from_raw(ptr);
    0
}

/// Set the receive callback function. This function will be called when a
/// frame is received.
#[no_mangle]
pub unsafe extern "C" fn zs_canfd_set_rx_callback(
    ptr: *mut CInterface,
    cb: Option<extern "C" fn(*const CFrame)>,
) -> i32 {
    let ci = &mut *ptr;
    ci.c_rx_cb = cb;
    0
}

/// Open the device. This must be called before any interaction with the
/// device (changing settings, starting communication).
#[no_mangle]
pub unsafe extern "C" fn zs_canfd_open(ptr: *mut CInterface) -> i32 {
    let i = match Interface::new() {
        Ok(i) => i,
        Err(_) => return -1,
    };
    let ci = &mut *ptr;
    ci.i = Some(i);
    0
}

/// Open the device. This must be called before any
/// interaction with the device by using the device
/// serial number (changing settings, starting communication).
#[no_mangle]
pub unsafe extern "C" fn zs_canfd_open_with_device_serial_number(ptr: *mut CInterface, dev_ser_num_ref: *const c_char) -> i32 {
    // Manually check for NULL case
    if dev_ser_num_ref.is_null() {
        return -1;
    }

    let dev_ser_num = match CStr::from_ptr(dev_ser_num_ref).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -1,
    };

    let i = match Interface::new_with_device_serial_number(dev_ser_num.clone()) {
        Ok(i) => i,
        Err(_) => return -1,
    };
    let ci = &mut *ptr;
    ci.i = Some(i);
    ci.dev_ser_num =  dev_ser_num;
    0
}

/// Close the device. After closing, no interaction with the device
/// can be performed.
#[no_mangle]
pub unsafe extern "C" fn zs_canfd_close(ptr: *mut CInterface) -> i32 {
    let ci = &mut *ptr;
    ci.i = None;
    0
}

/// Start CAN communication. This will enable all configured CAN channels.
///
/// This function starts a thread which will call the registered callback
/// when a frame is received.
#[no_mangle]
pub unsafe extern "C" fn zs_canfd_start(
    ptr: *mut CInterface,
    channel: u8,
) -> i32 {
    let ci = &mut *ptr;

    let cb = ci.c_rx_cb;
    let c_string = CString::new(ci.dev_ser_num.clone()).expect("CString::new failed");
    match &mut ci.i {
        Some(i) => i
            .start(channel as usize, move |f: Frame| {
                match cb {
                    None => {}
                    Some(cb) => {
                        cb(&CFrame::from_frame(f, c_string.clone()));
                    }
                };
            })
            .expect("failed to start device"),
        None => return -1,
    };
    0
}

/// Stop CAN communication. This will stop all configured CAN channels.
#[no_mangle]
pub unsafe extern "C" fn zs_canfd_stop(
    ptr: *mut CInterface,
    channel: u8,
) -> i32 {
    let ci = &mut *ptr;
    match &mut ci.i {
        Some(i) => i.stop(channel as usize).expect("failed to stop device"),
        None => return -1,
    }
    0
}

/// Transmit a frame. Can only be called if the device is running.
#[no_mangle]
pub unsafe extern "C" fn zs_canfd_transmit(ptr: *mut CInterface, cf: CFrame) -> i32 {
    let ci = &mut *ptr;
    let f = Frame {
        channel: cf.channel, //cf.channel,
        can_id: cf.id,
        can_dlc: cf.dlc,
        data: cf.data.to_vec(),
        ext: cf.ext > 0,
        fd: cf.fd > 0,
        brs: cf.brs > 0,
        esi: cf.esi > 0,
        loopback: false,
        rtr: cf.rtr > 0,
        err: cf.err > 0,
        sw_timestamp: None,
        hw_timestamp: 0,
    };
    match &mut ci.i {
        Some(i) => i.send(f).expect("failed to transmit frame"),
        None => return -1,
    };
    0
}

/// Sets the bitrate for a channel to the given value in bits per second.
#[no_mangle]
pub unsafe extern "C" fn zs_canfd_set_bitrate(
    ptr: *mut CInterface,
    channel: u8,
    bitrate: u32,
) -> i32 {
    let ci = &mut *ptr;
    match &mut ci.i {
        Some(i) => i
            .set_bitrate(channel as usize, bitrate)
            .expect("failed to set bitrate"),
        None => return -1,
    }
    0
}

/// Sets the data bitrate for a channel to the given value in bits per second.
#[no_mangle]
pub unsafe extern "C" fn zs_canfd_set_data_bitrate(
    ptr: *mut CInterface,
    channel: u8,
    bitrate: u32,
) -> i32 {
    let ci = &mut *ptr;
    match &mut ci.i {
        Some(i) => i
            .set_data_bitrate(channel as usize, bitrate)
            .expect("failed to set bitrate"),
        None => return -1,
    }
    0
}

/// Enable or disable a channel.
#[no_mangle]
pub unsafe extern "C" fn zs_canfd_set_enabled(
    ptr: *mut CInterface,
    channel: u8,
    enabled: u8,
) -> i32 {
    let ci = &mut *ptr;
    match &mut ci.i {
        Some(i) => i
            .set_enabled(channel as usize, enabled > 0)
            .expect("failed to enable channel"),
        None => return -1,
    }
    0
}

/// Enable or disable bus monitoring mode for a channel. When enabled, channel
/// will not transmit frames or acknoweldgements.
#[no_mangle]
pub unsafe extern "C" fn zs_canfd_set_monitor(
    ptr: *mut CInterface,
    channel: u8,
    enabled: u8,
) -> i32 {
    let ci = &mut *ptr;
    match &mut ci.i {
        Some(i) => i
            .set_monitor(channel as usize, enabled > 0)
            .expect("failed to set monitoring mode"),
        None => return -1,
    }
    0
}

/// Enable or disable hardware loopback for a channel. This will cause sent
/// frames to be received. This mode is mostly intended for device testing.
#[no_mangle]
pub unsafe extern "C" fn zs_canfd_set_hw_loopback(
    ptr: *mut CInterface,
    channel: u8,
    enabled: u8,
) -> i32 {
    let ci = &mut *ptr;
    match &mut ci.i {
        Some(i) => i
            .set_loopback(channel as usize, enabled > 0)
            .expect("failed to enable channel"),
        None => return -1,
    }
    0
}

/// Get the number of CAN channels the device has.
///
/// Returns the number of channels or a negative error code on failure.
#[no_mangle]
pub unsafe extern "C" fn zs_canfd_get_channel_count(ptr: *mut CInterface) -> i32 {
    let ci = &mut *ptr;
    match &mut ci.i {
        Some(i) => i.channels() as i32,
        None => -1,
    }
}

/// Enable or disable FD mode for a channel.
#[no_mangle]
pub unsafe extern "C" fn zs_canfd_set_fd(
    ptr: *mut CInterface,
    channel: u8,
    enabled: u8,
) -> i32 {
    let ci = &mut *ptr;
    match &mut ci.i {
        Some(i) => i
            .set_fd(channel as usize, enabled > 0)
            .expect("failed to set fd mode"),
        None => return -1,
    }
    0
}

/// Get available devices
#[no_mangle]
pub unsafe extern "C" fn zs_canfd_get_available_devices_count() -> i32 {
    let mut context = mem::MaybeUninit::<*mut libusb_context>::uninit();

    let ctx = match unsafe { libusb_init(context.as_mut_ptr()) } {
        LIBUSB_SUCCESS => unsafe { context.assume_init() },
        _ => return 0,
    };

    let count = libusb_helper::LibUsbHelperDevice::new(ctx, USB_VID, USB_PID, String::from("")).get_available_devices();

    match count {
        count if count > 0 => count,
        _ => 0
    }
}

/// Get available devices serial numbers
#[no_mangle]
pub unsafe extern "C" fn zs_canfd_get_available_devices_serial_number() -> *const c_char {
     let mut context = mem::MaybeUninit::<*mut libusb_context>::uninit();

    let ctx = match unsafe { libusb_init(context.as_mut_ptr()) } {
        LIBUSB_SUCCESS => unsafe { context.assume_init() },
        _ => return null_mut(),
    };

    let serial_nums = libusb_helper::LibUsbHelperDevice::new(ctx, USB_VID, USB_PID, String::from("")).get_available_devices_serial_numbers();
    let mut comb_str = String::new();
    let spliter = "|";

    // Instead of passing a list of serial numbers, pass the serial numbers as a pipe-delimited string.
    // Example: 123|124|12X. If no serial numbers are provided, pass an empty string ("").
    for s in serial_nums.iter() {
        comb_str = comb_str.to_owned() + spliter + s;
    }

    let c_string = CString::new(comb_str).expect("CString::new failed");
    c_string.into_raw()
}

/// To free the 'c_string' created from 'zs_canfd_get_available_devices_serial_number()'
#[no_mangle]
pub unsafe extern "C" fn free_string(ptr: *const c_char) -> i32 {
    // Take the ownership back to rust and drop the owner
    let _ = CString::from_raw(ptr as *mut _);
    0
}
