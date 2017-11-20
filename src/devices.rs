use Object;
use Device;

use std::ops::Deref;

use coremidi_sys::{
    MIDIGetNumberOfDevices, MIDIGetDevice, ItemCount
};

impl Device {
    pub fn from_index(index: usize) -> Device {
        let device_ref = unsafe { MIDIGetDevice(index as ItemCount) };
        Device { object: Object(device_ref) }
    }
}

impl Deref for Device {
    type Target = Object;

    fn deref(&self) -> &Object {
        &self.object
    }
}


pub struct Devices;

impl Devices {
    pub fn count() -> usize {
        unsafe { MIDIGetNumberOfDevices() as usize }
    }
}

impl IntoIterator for Devices {
    type Item = Device;
    type IntoIter = DevicesIterator;

    fn into_iter(self) -> Self::IntoIter {
        DevicesIterator { index: 0, count: Self::count() }
    }
}

pub struct DevicesIterator {
    index: usize,
    count: usize
}

impl Iterator for DevicesIterator {
    type Item = Device;

    fn next(&mut self) -> Option<Device> {
        if self.index < self.count {
            let source = Some(Device::from_index(self.index));
            self.index += 1;
            source
        } else {
            None
        }
    }
}
