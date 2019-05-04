use std::marker::PhantomData;
use std::mem;

use libusb::*;

use config_descriptor::{self, ConfigDescriptor};
use context::Context;
use device_descriptor::{self, DeviceDescriptor};
use device_handle::{self, DeviceHandle};
use fields::{self, Speed};

/// A reference to a USB device.
pub struct Device<'a> {
    context: PhantomData<&'a Context>,
    device: *mut libusb_device,
}

impl<'a> Drop for Device<'a> {
    /// Releases the device reference.
    fn drop(&mut self) {
        unsafe {
            libusb_unref_device(self.device);
        }
    }
}

unsafe impl<'a> Send for Device<'a> {}
unsafe impl<'a> Sync for Device<'a> {}

impl<'a> Device<'a> {
    /// Reads the device descriptor.
    pub fn device_descriptor(&self) -> ::Result<DeviceDescriptor> {
        let mut descriptor: libusb_device_descriptor = unsafe { mem::uninitialized() };

        // since libusb 1.0.16, this function always succeeds
        try_unsafe!(libusb_get_device_descriptor(self.device, &mut descriptor));

        Ok(device_descriptor::from_libusb(descriptor))
    }

    /// Reads a configuration descriptor.
    pub fn config_descriptor(&self, config_index: u8) -> ::Result<ConfigDescriptor> {
        let mut config: *const libusb_config_descriptor = unsafe { mem::uninitialized() };

        try_unsafe!(libusb_get_config_descriptor(
            self.device,
            config_index,
            &mut config
        ));

        Ok(unsafe { config_descriptor::from_libusb(config) })
    }

    /// Reads the configuration descriptor for the current configuration.
    pub fn active_config_descriptor(&self) -> ::Result<ConfigDescriptor> {
        let mut config: *const libusb_config_descriptor = unsafe { mem::uninitialized() };

        try_unsafe!(libusb_get_active_config_descriptor(
            self.device,
            &mut config
        ));

        Ok(unsafe { config_descriptor::from_libusb(config) })
    }

    /// Returns the number of the bus that the device is connected to.
    pub fn bus_number(&self) -> u8 {
        unsafe { libusb_get_bus_number(self.device) }
    }

    /// Returns the devices's port number on the bus that it's connected to.
    pub fn port_number(&self) -> u8 {
        unsafe { libusb_get_port_number(self.device) }
    }

    /// Returns the devices's port numbers on the bus that it's connected to.
    pub fn port_numbers(&self) -> Vec<u8> {
        let mut array: [u8; 7] = [0; 7];
        let array_ptr: *mut u8 = &mut array[0] as *mut u8;
        unsafe {
            let actual_len =
                libusb_get_port_numbers(self.device, array_ptr, array.len() as i32) as usize;
            let mut nums = Vec::new();
            for num in &array[..actual_len] {
                nums.push(num.clone());
            }
            nums
        }
    }

    /// Returns the device's address on the bus that it's connected to.
    pub fn address(&self) -> u8 {
        unsafe { libusb_get_device_address(self.device) }
    }

    /// Returns the device's connection speed.
    pub fn speed(&self) -> Speed {
        fields::speed_from_libusb(unsafe { libusb_get_device_speed(self.device) })
    }

    /// Opens the device.
    pub fn open(&self) -> ::Result<DeviceHandle<'a>> {
        let mut handle: *mut libusb_device_handle = unsafe { mem::uninitialized() };

        try_unsafe!(libusb_open(self.device, &mut handle));

        Ok(unsafe { device_handle::from_libusb(self.context, handle) })
    }

    /// Returns the parent device
    pub fn parent(&self) -> Option<Self> {
        unsafe {
            let parent = libusb_get_parent(self.device);
            if parent.is_null() {
                None
            } else {
                Some(Self {
                    context: self.context,
                    device: parent,
                })
            }
        }
    }
}

#[doc(hidden)]
pub unsafe fn from_libusb<'a>(
    context: PhantomData<&'a Context>,
    device: *mut libusb_device,
) -> Device<'a> {
    libusb_ref_device(device);

    Device {
        context: context,
        device: device,
    }
}
