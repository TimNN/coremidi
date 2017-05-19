#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]

use core_foundation_sys::base::OSStatus;
use core_foundation_sys::string::CFStringRef;

use coremidi_sys::{
    MIDIClientRef, MIDIEndpointRef, MIDIPortRef,
};

pub type MIDIReadProc =
    ::std::option::Option<extern "C" fn(pktlist: *const MIDIPacketList,
                                        readProcRefCon: *mut ::libc::c_void,
                                        srcConnRefCon: *mut ::libc::c_void)
                              -> ()>;

// Should only be used in a pointer
#[repr(C)]
pub struct MIDIPacketList(u8);

extern "C" {
    pub fn MIDISend(port: MIDIPortRef, dest: MIDIEndpointRef,
                    pktlist: *const MIDIPacketList) -> OSStatus;

    pub fn MIDIReceived(src: MIDIEndpointRef, pktlist: *const MIDIPacketList) -> OSStatus;

    pub fn MIDIInputPortCreate(client: MIDIClientRef, portName: CFStringRef,
                               readProc: MIDIReadProc,
                               refCon: *mut ::libc::c_void,
                               outPort: *mut MIDIPortRef) -> OSStatus;

    pub fn MIDIDestinationCreate(client: MIDIClientRef, name: CFStringRef,
                                readProc: MIDIReadProc,
                                refCon: *mut ::libc::c_void,
                                outDest: *mut MIDIEndpointRef) -> OSStatus;
}
