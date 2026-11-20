use libc::{c_int, c_uchar};
use libusb1_sys;
use libusb1_sys::libusb_device;
use std::mem;
use std::ptr;
use std::slice;

pub struct LibUsbHelperDevice {
    vendor_id: u16,
    product_id: u16,
    serial_number: String,
    ctx: *mut libusb1_sys::libusb_context,
}

impl LibUsbHelperDevice {
    pub fn new(ctx: *mut libusb1_sys::libusb_context, vendor_id: u16, product_id: u16, serial_num: String) -> LibUsbHelperDevice {
        LibUsbHelperDevice {
            serial_number: serial_num,
            vendor_id,
            product_id,
            ctx,
        }
    }

    // Return the USB serial number as string
    fn get_usb_string_descriptor(&mut self, handle: *mut libusb1_sys::libusb_device_handle, desc_index: u8) -> Option<String> {
        if handle.is_null() || (0 == desc_index) {
            return None;
        }

        let mut vec = Vec::<u8>::with_capacity(256);
        let ptr = vec.as_mut_ptr();

        let len = unsafe {
            libusb1_sys::libusb_get_string_descriptor_ascii(
                handle,
                desc_index,
                ptr as *mut c_uchar,
                vec.capacity() as c_int,
            )
        };

        if len > 0 {
            unsafe { vec.set_len(len as usize) };
            String::from_utf8(vec).ok()
        } else {
            None
        }
    }

    // Return the USB device handle
    pub fn open_device_with_vid_pid_serial(&mut self) -> *mut libusb1_sys::libusb_device_handle {
        // Creates a null mutable raw pointer.
        let mut ret_handle: *mut libusb1_sys::libusb_device_handle = ptr::null_mut();

        // <MaybeUninit> used for avoid unnecessary initialization
        let mut listdev: mem::MaybeUninit<*const *mut libusb1_sys::libusb_device> = mem::MaybeUninit::<*const *mut libusb1_sys::libusb_device>::uninit();  //list

        // Get the available USB devices
        let available_devices_count = unsafe { libusb1_sys::libusb_get_device_list(self.ctx, listdev.as_mut_ptr())};
         if available_devices_count < 0 {
            return ret_handle ;
         }
        // Convert the MaybeUninit to an libusb_device, assuming it's initialized
        let device_list = unsafe { listdev.assume_init() };

        // Raw pointer obtained through libusb (FFI) convert to &mut [T]
        let devs = unsafe { slice::from_raw_parts(device_list, available_devices_count as usize) };
        // Iterate each USB device from devs
        for dev in devs {
            let mut desc = mem::MaybeUninit::<libusb1_sys::libusb_device_descriptor>::uninit();
            let mut handle: *mut libusb1_sys::libusb_device_handle = ptr::null_mut();
            let descriptor: Option<libusb1_sys::libusb_device_descriptor> =
            // Get device descriptor
            match unsafe { libusb1_sys::libusb_get_device_descriptor(*dev, desc.as_mut_ptr()) } {
                0 => {
                    Some(unsafe { desc.assume_init() })
                },
                _  => None,
            };

            // Check if &descriptor have data if yes &descriptor assign  Some(descriptor) then block will run otherwise skip
            if let Some(descriptor) = &descriptor {
                // Compare VID and PID
                if (descriptor.idVendor == self.vendor_id) && (descriptor.idProduct == self.product_id) {
                    unsafe {
                        // Open USB device
                        if  libusb1_sys::libusb_open(*dev, &mut handle)  < 0 {
                            // Couldn't open device, some information will be missing
                            continue;
                        }

                        let mut dev_serial =  String::new();
                        if descriptor.iSerialNumber > 0 {
                            // Get the USB serial number
                            if let Some(serial) = self.get_usb_string_descriptor(handle, descriptor.iSerialNumber) {
                                dev_serial = serial;
                            }
                        }
                        // Compare serial number
                        if dev_serial == self.serial_number {
                            ret_handle = handle;
                            // Free the device list memory
                            libusb1_sys::libusb_free_device_list(device_list,1);
                            break;
                        } else {
                            // When match failed close the object
                            if !handle.is_null() {
                                libusb1_sys::libusb_close(handle);
                            }
                        }
                    }
                }
            }
        }
        ret_handle
    }

    // Return the available device counts
    pub fn get_available_devices(&mut self) -> i32 {
        // Counter count the device
        let mut device_count = 0;
        let mut list_dev: *const *mut libusb1_sys::libusb_device = std::ptr::null();

        // Get the available USB devices
        let available_devices_count = unsafe { libusb1_sys::libusb_get_device_list(self.ctx, &mut list_dev)};
        if available_devices_count < 0 {
            return -1;
        }

        // Raw pointer obtained through libusb (FFI) convert to &mut [T]
        let devs = unsafe { slice::from_raw_parts(list_dev, available_devices_count as usize) };

        // Iterate each USB device from devs
        for dev in devs {
            let mut desc = mem::MaybeUninit::<libusb1_sys::libusb_device_descriptor>::uninit();
            let descriptor: Option<libusb1_sys::libusb_device_descriptor> =
            // Get device descriptor
            match unsafe { libusb1_sys::libusb_get_device_descriptor(*dev, desc.as_mut_ptr()) } {
                0 => {
                    Some(unsafe { desc.assume_init() })
                },
                _  => None,
            };

            // Check if &descriptor have data if yes &descriptor assign  Some(descriptor) then block will run otherwise skip
            if let Some(descriptor) = &descriptor {
                // Compare VID and PID
                if (descriptor.idVendor == self.vendor_id) && (descriptor.idProduct == self.product_id) {
                    device_count += 1;
                }
            }
        }
        unsafe { libusb1_sys::libusb_free_device_list(list_dev, 1) };
        device_count
    }

    fn get_device_serial_number(&mut self, dev : &*mut libusb_device) -> String {
        let mut desc = mem::MaybeUninit::<libusb1_sys::libusb_device_descriptor>::uninit();
        let mut handle: *mut libusb1_sys::libusb_device_handle = ptr::null_mut();
        let mut dev_serial =  String::new();
        let descriptor: Option<libusb1_sys::libusb_device_descriptor> =
        // Get device descriptor
        match unsafe { libusb1_sys::libusb_get_device_descriptor(*dev, desc.as_mut_ptr()) } {
            0 => {
                Some(unsafe { desc.assume_init() })
            },
            _  => None,
        };

        // Check if &descriptor have data if yes &descriptor assign  Some(descriptor) then block will run otherwise skip
        if let Some(descriptor) = &descriptor {
            // Compare VID and PID
            if (descriptor.idVendor == self.vendor_id) && (descriptor.idProduct == self.product_id) {
                unsafe {
                    // Open USB device
                    if  libusb1_sys::libusb_open(*dev, &mut handle)  < 0 {
                        // Couldn't open device, some information will be missing
                        return dev_serial;
                    }

                    if descriptor.iSerialNumber > 0 {
                        // Get the USB serial number
                        if let Some(serial) = self.get_usb_string_descriptor(handle, descriptor.iSerialNumber) {
                            dev_serial = serial;
                        }
                    }

                    if !handle.is_null() {
                        libusb1_sys::libusb_close(handle);
                    }
                }
            }
        }
        dev_serial
    }

    // Return the available devices serial numbers
    pub fn get_available_devices_serial_numbers(&mut self) -> Vec<String> {
        let mut serial_nums : Vec<String> =  Vec::new();

        // <MaybeUninit> used for avoid unnecessary initialization
        let mut listdev: mem::MaybeUninit<*const *mut libusb1_sys::libusb_device> = mem::MaybeUninit::<*const *mut libusb1_sys::libusb_device>::uninit();  //list

        // Get the available USB devices
        let available_devices_count = unsafe { libusb1_sys::libusb_get_device_list(self.ctx, listdev.as_mut_ptr())};
        if available_devices_count < 0 {
            return serial_nums;
         }
        // Convert the MaybeUninit to an libusb_device, assuming it's initialized
        let device_list = unsafe { listdev.assume_init() };

        // Raw pointer obtained through libusb (FFI) convert to &mut [T]
        let devs = unsafe { slice::from_raw_parts(device_list, available_devices_count as usize) };

        // Iterate each USB device from devs
        for dev in devs {
            let dev_serial = self.get_device_serial_number(dev);
            if dev_serial.is_empty() {
                continue;
            }
            serial_nums.push(dev_serial);
        }

        unsafe {
            // Free the device list memory
            libusb1_sys::libusb_free_device_list(device_list,1);
        }
        serial_nums
    }
}
