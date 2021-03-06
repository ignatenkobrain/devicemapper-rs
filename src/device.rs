// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;
use libc::{dev_t, major, makedev, minor};

/// A struct containing the device's major and minor numbers
///
/// Also allows conversion to/from a single 64bit dev_t value.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Device {
    /// Device major number
    pub major: u32,
    /// Device minor number
    pub minor: u32,
}

/// Display format is the device number in "<major>:<minor>" format
impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.major, self.minor)
    }
}

impl From<dev_t> for Device {
    fn from(val: dev_t) -> Device {
        Device {
            major: unsafe { major(val) },
            minor: unsafe { minor(val) },
        }
    }
}

impl From<Device> for dev_t {
    fn from(dev: Device) -> dev_t {
        unsafe { makedev(dev.major, dev.minor) }
    }
}

/// The Linux kernel's kdev_t encodes major/minor values as mmmM MMmm.
impl Device {
    /// Make a Device from a kdev_t.
    pub fn from_kdev_t(val: u32) -> Device {
        Device {
            major: (val & 0xf_ff00) >> 8,
            minor: (val & 0xff) | ((val >> 12) & 0xf_ff00),
        }
    }

    /// Convert to a kdev_t. Return None if values are not expressible as a
    /// kdev_t.
    pub fn to_kdev_t(&self) -> Option<u32> {
        if self.major > 0xfff || self.minor > 0xf_ffff {
            return None;
        }

        Some((self.minor & 0xff) | (self.major << 8) | ((self.minor & !0xff) << 12))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    /// Verify conversion is correct both ways
    pub fn test_dev_t_conversion() {
        let test_devt_1: dev_t = 0xabcdef1234567890;

        let dev1 = Device::from(test_devt_1);
        // Default glibc dev_t encoding is MMMM Mmmm mmmM MMmm. I guess if
        // we're on a platform where non-default is used, we'll fail.
        assert_eq!(dev1.major, 0xabcde678);
        assert_eq!(dev1.minor, 0xf1234590);

        let test_devt_2: dev_t = dev_t::from(dev1);
        assert_eq!(test_devt_1, test_devt_2);
    }

    #[test]
    /// Verify conversion is correct both ways
    pub fn test_kdev_t_conversion() {
        let test_devt_1: u32 = 0x12345678;

        let dev1 = Device::from_kdev_t(test_devt_1);
        // Default kernel kdev_t "huge" encoding is mmmM MMmm.
        assert_eq!(dev1.major, 0x456);
        assert_eq!(dev1.minor, 0x12378);

        let test_devt_2: u32 = dev1.to_kdev_t().unwrap();
        assert_eq!(test_devt_1, test_devt_2);

        // a Device inexpressible as a kdev_t
        let dev2 = Device::from(0xabcdef1234567890);
        assert_eq!(dev2.to_kdev_t(), None);
    }
}
