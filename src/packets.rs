use coremidi_sys_ext::{
    MIDIPacketList,
};

use std::fmt;
use std::ptr;
use std::slice;
use std::marker::PhantomData;

pub type Timestamp = u64;

// From the CoreMIDI headers:
//
// A Packet consists of a timestamp(u64), a length(u16) and a variable amount of
// data, which can be either a) one (or part of one) SysEx message or multiple
// complete normal messages. Running status is not allowed.
//
// A PacketList consists of the numberOfPackets(u32) followed by the packet
// data.
//
// Both structs are marked with `#pragma pack(push, 4)`


/// A [list of MIDI events](https://developer.apple.com/reference/coremidi/midipacketlist) being received from, or being sent to, one endpoint.
///
#[derive(Copy, Clone)]
pub struct PacketListRef<'a> {
    data: *const u8,
    _lt: PhantomData<&'a u8>,
}

/// A collection of simultaneous MIDI events.
/// See [MIDIPacket](https://developer.apple.com/reference/coremidi/midipacket).
///
#[derive(Copy, Clone)]
pub struct PacketRef<'a> {
    data: *const u8,
    _lt: PhantomData<&'a u8>,
}

impl<'a> PacketRef<'a> {

    // The loads here may be entirely unaligned on X86 or 4-byte aligned on ARM.
    // Avoid undefined behavior by using `read_unaligned` even though a normal
    // load should not cause problems as far as I can tell.

    /// Get the packet timestamp.
    ///
    #[inline(always)]
    pub fn timestamp(&self) -> Timestamp {
        unsafe { ptr::read_unaligned(self.data as *const _) }
    }

    /// Get the number of data bytes in thie packet.
    ///
    #[inline(always)]
    pub fn data_length(&self) -> u16 {
        unsafe { ptr::read_unaligned(self.data.offset(8) as *const _) }
    }

    /// Get the packet data. This method just gives raw MIDI bytes. You would need another
    /// library to decode them and work with higher level events.
    ///
    ///
    /// The following example:
    ///
    /// ```
    /// let packet_list = &coremidi::PacketBuffer::from_data(0, vec![0x90, 0x40, 0x7f]);
    /// for packet in packet_list.iter() {
    ///   for byte in packet.data() {
    ///     print!(" {:x}", byte);
    ///   }
    /// }
    /// ```
    ///
    /// will print:
    ///
    /// ```text
    ///  90 40 7f
    /// ```
    #[inline(always)]
    pub fn data(&self) -> &'a [u8] {
        unsafe { slice::from_raw_parts(self.data.offset(10), self.data_length() as usize) }
    }

    #[inline(always)]
    unsafe fn next(&self) -> PacketRef<'a> {
        let unadjusted = self.data.offset(10 + self.data_length() as isize);

        #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
        return PacketRef {
            data: ((unadjusted as usize + 3) & !3) as *const _,
            _lt: PhantomData,
        };

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        return PacketRef {
            data: unadjusted,
            _lt: PhantomData,
        };
    }
}

impl<'a> fmt::Debug for PacketRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result = write!(f, "Packet(ptr={:x}, ts={:016x}, data=[",
                            self.data as usize, self.timestamp() as u64);
        let result = self.data().iter().enumerate().fold(result, |prev_result, (i, b)| {
            match prev_result {
                Err(err) => Err(err),
                Ok(()) => {
                    let sep = if i > 0 { ", " } else { "" };
                    write!(f, "{}{:02x}", sep, b)
                }
            }
        });
        result.and_then(|_| write!(f, "])"))
    }
}

impl<'a> fmt::Display for PacketRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result = write!(f, "{:016x}:", self.timestamp());
        self.data().iter().fold(result, |prev_result, b| {
            match prev_result {
                Err(err) => Err(err),
                Ok(()) => write!(f, " {:02x}", b)
            }
        })
    }
}

impl<'a> PacketListRef<'a> {
    #[inline(always)]
    pub unsafe fn from_ptr(ptr: *const MIDIPacketList) -> PacketRef<'a> {
        PacketRef {
            data: ptr as *const _,
            _lt: PhantomData,
        }
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *const MIDIPacketList {
        self.data as *const _
    }

    /// Get the number of packets in the list.
    ///
    #[inline(always)]
    pub fn length(&self) -> u32 {
        // PacketList should always be 4 byte aligned
        unsafe { *(self.data as *const _) }
    }

    /// Get an iterator for the packets in the list.
    ///
    #[inline(always)]
    pub fn iter(&self) -> PacketListIterator<'a> {
        PacketListIterator {
            remaining: self.length(),
            packet_ref: PacketRef { data: unsafe { self.data.offset(4) }, _lt: PhantomData }
        }
    }
}

impl<'a> fmt::Debug for PacketListRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result = write!(f, "PacketList(ptr={:x}, packets=[", self.data as usize);
        self.iter().enumerate().fold(result, |prev_result, (i, packet)| {
            match prev_result {
                Err(err) => Err(err),
                Ok(()) => {
                    let sep = if i != 0 { ", " } else { "" };
                    write!(f, "{}{:?}", sep, packet)
                }
            }
        }).and_then(|_| write!(f, "])"))
    }
}

impl<'a> fmt::Display for PacketListRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result = write!(f, "PacketList(len={})", self.length());
        self.iter().fold(result, |prev_result, packet| {
            match prev_result {
                Err(err) => Err(err),
                Ok(()) => write!(f, "\n  {}", packet)
            }
        })
    }
}

pub struct PacketListIterator<'a> {
    remaining: u32,
    packet_ref: PacketRef<'a>,
}

impl<'a> Iterator for PacketListIterator<'a> {
    type Item = PacketRef<'a>;

    #[inline]
    fn next(&mut self) -> Option<PacketRef<'a>> {
        if self.remaining > 0 {
            self.remaining -= 1;
            let packet_ref = self.packet_ref;
            self.packet_ref = unsafe { self.packet_ref.next() };
            Some(packet_ref)
        }
        else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use coremidi_sys::MIDITimeStamp;
    use coremidi_sys_ext::MIDIPacketList;
    use PacketList;
    use PacketBuffer;

    #[test]
    pub fn packet_buffer_new() {
        let packet_buf = PacketBuffer::new();
        assert_eq!(packet_buf.data.len(), 4);
        assert_eq!(packet_buf.data, vec![0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    pub fn packet_buffer_with_data() {
        let packet_buf = PacketBuffer::new()
            .with_data(0x0102030405060708 as MIDITimeStamp, vec![0x90u8, 0x40, 0x7f]);
        assert_eq!(packet_buf.data.len(), 17);
        // FIXME This is platform endianess dependent
        assert_eq!(packet_buf.data, vec![
            0x01, 0x00, 0x00, 0x00,
            0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01,
            0x03, 0x00,
            0x90, 0x40, 0x7f]);
    }

    #[test]
    fn packet_buffer_deref() {
        let packet_buf = PacketBuffer::new();
        let packet_list: &PacketList = &packet_buf;
        assert_eq!(packet_list.0, &packet_buf.data[0] as *const _ as *const MIDIPacketList);
    }

    #[test]
    fn packet_list_length() {
        let packet_buf = PacketBuffer::new()
            .with_data(0, vec![0x90u8, 0x40, 0x7f])
            .with_data(0, vec![0x91u8, 0x40, 0x7f])
            .with_data(0, vec![0x80u8, 0x40, 0x7f])
            .with_data(0, vec![0x81u8, 0x40, 0x7f]);
        assert_eq!(packet_buf.length(), 4);
    }
}
