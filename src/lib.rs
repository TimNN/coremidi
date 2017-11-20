#![crate_name = "coremidi"]
#![crate_type = "lib"]
#![doc(html_root_url = "https://chris-zen.github.io/coremidi/")]

/*!
This is a [CoreMIDI](https://developer.apple.com/reference/coremidi) library for Rust built on top of the low-level bindings [coremidi-sys](https://github.com/jonas-k/coremidi-sys).
CoreMIDI is a Mac OSX framework that provides APIs for communicating with MIDI (Musical Instrument Digital Interface) devices, including hardware keyboards and synthesizers.

This library preserves the fundamental concepts behind the CoreMIDI framework, while being Rust idiomatic. This means that if you already know CoreMIDI, you will find very easy to start using it.

Please see the [examples](examples) for an idea on how it looks like, but if you are eager to see an example, this is how you would send some note:

```rust,no_run
extern crate coremidi;
use std::time::Duration;
use std::thread;
let client = coremidi::Client::new("example-client").unwrap();
let output_port = client.output_port("example-port").unwrap();
let destination = coremidi::Destination::from_index(0);
let note_on = coremidi::PacketBuffer::from_data(0, vec![0x90, 0x40, 0x7f]);
let note_off = coremidi::PacketBuffer::from_data(0, vec![0x80, 0x40, 0x7f]);
output_port.send(&destination, note_on.as_ref()).unwrap();
thread::sleep(Duration::from_millis(1000));
output_port.send(&destination, note_off.as_ref()).unwrap();
```

If you are looking for a portable MIDI library then you can look into:

- [portmidi-rs](https://github.com/musitdev/portmidi-rs)
- [midir](https://github.com/Boddlnagg/midir)

For handling low level MIDI data you may look into:

- [midi-rs](https://github.com/samdoshi/midi-rs)
- [rimd](https://github.com/RustAudio/rimd)

**Please note that this is a work in progress project !**

*/

extern crate core_foundation_sys;
extern crate core_foundation;
extern crate coremidi_sys;
extern crate libc;

use core_foundation_sys::base::OSStatus;

use coremidi_sys::{
    MIDIObjectRef, MIDIFlushOutput, MIDIRestart
};

/// A [MIDI Object](https://developer.apple.com/reference/coremidi/midiobjectref).
///
/// The base class of many CoreMIDI objects.
///
#[derive(PartialEq)]
pub struct Object(MIDIObjectRef);

/// A [MIDI client](https://developer.apple.com/reference/coremidi/midiclientref).
///
/// An object maintaining per-client state.
///
/// A simple example to create a Client:
///
/// ```rust,no_run
/// let client = coremidi::Client::new("example-client").unwrap();
/// ```
pub struct Client {
    // Order is important, object needs to be dropped first
    object: Object,
    // Never used once set but needs to stay alive.
    _callback: BoxedCallback<Box<FnMut(&Notification)>>,
}

// A lifetime-managed wrapper for callback functions
#[derive(PartialEq)]
struct BoxedCallback<T>(*mut T);

impl<T> BoxedCallback<T> {
    fn new(t: T) -> BoxedCallback<T> {
        BoxedCallback(Box::into_raw(Box::new(t)))
    }

    fn null() -> BoxedCallback<T> {
        BoxedCallback(::std::ptr::null_mut())
    }

    fn raw_ptr(&mut self) -> *mut ::libc::c_void {
        self.0 as *mut ::libc::c_void
    }

    // must not be null
    unsafe fn call_from_raw_ptr<X>(raw_ptr: *mut ::libc::c_void, arg: X) {
        let callback = &mut *(raw_ptr as *mut Box<FnMut(X)>);
        callback(arg);
    }
}

impl<T> Drop for BoxedCallback<T> {
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_null() {
                let _ = Box::from_raw(self.0);
            }
        }
    }
}

/// A MIDI connection port owned by a client.
/// See [MIDIPortRef](https://developer.apple.com/reference/coremidi/midiportref).
///
/// Ports can't be instantiated directly, but through a client.
///
#[derive(Debug)]
pub struct Port { object: Object }

/// An output [MIDI port](https://developer.apple.com/reference/coremidi/midiportref) owned by a client.
///
/// A simple example to create an output port and send a MIDI event:
///
/// ```rust,no_run
/// let client = coremidi::Client::new("example-client").unwrap();
/// let output_port = client.output_port("example-port").unwrap();
/// let destination = coremidi::Destination::from_index(0);
/// let packets = coremidi::PacketBuffer::from_data(0, vec![0x90, 0x40, 0x7f]);
/// output_port.send(&destination, packets.as_ref()).unwrap();
/// ```
#[derive(Debug)]
pub struct OutputPort { port: Port }

/// An input [MIDI port](https://developer.apple.com/reference/coremidi/midiportref) owned by a client.
///
/// A simple example to create an input port:
///
/// ```rust,no_run
/// let client = coremidi::Client::new("example-client").unwrap();
/// let input_port = client.input_port("example-port", |packet_list| println!("{}", packet_list)).unwrap();
/// let source = coremidi::Source::from_index(0);
/// input_port.connect_source(&source);
/// ```
pub struct InputPort {
    // Note: the order is important here, port needs to be dropped first
    port: Port,
    // Never used once set but needs to stay alive.
    _callback: BoxedCallback<Box<FnMut(PacketListRef)>>,
}

/// A MIDI source or source, owned by an entity.
/// See [MIDIEndpointRef](https://developer.apple.com/reference/coremidi/midiendpointref).
///
/// You don't need to create an endpoint directly, instead you can create system sources and sources or virtual ones from a client.
///
#[derive(Debug)]
pub struct Endpoint { object: Object }

/// A [MIDI source](https://developer.apple.com/reference/coremidi/midiendpointref) owned by an entity.
///
/// A source can be created from an index like this:
///
/// ```rust,no_run
/// let source = coremidi::Destination::from_index(0);
/// println!("The source at index 0 has display name '{}'", source.display_name().unwrap());
/// ```
///
#[derive(Debug)]
pub struct Destination { endpoint: Endpoint }

/// A [MIDI source](https://developer.apple.com/reference/coremidi/midiendpointref) owned by an entity.
///
/// A source can be created from an index like this:
///
/// ```rust,no_run
/// let source = coremidi::Source::from_index(0);
/// println!("The source at index 0 has display name '{}'", source.display_name().unwrap());
/// ```
///
#[derive(Debug)]
pub struct Source { endpoint: Endpoint }

/// A [MIDI virtual source](https://developer.apple.com/reference/coremidi/1495212-midisourcecreate) owned by a client.
///
/// A virtual source can be created like:
///
/// ```rust,no_run
/// let client = coremidi::Client::new("example-client").unwrap();
/// let source = client.virtual_source("example-source").unwrap();
/// ```
///
#[derive(Debug)]
pub struct VirtualSource { endpoint: Endpoint }

/// A [MIDI virtual destination](https://developer.apple.com/reference/coremidi/1495347-mididestinationcreate) owned by a client.
///
/// A virtual destination can be created like:
///
/// ```rust,no_run
/// let client = coremidi::Client::new("example-client").unwrap();
/// client.virtual_destination("example-destination", |packet_list| println!("{}", packet_list)).unwrap();
/// ```
///
pub struct VirtualDestination {
    // Note: the order is important here, endpoint needs to be dropped first
    endpoint: Endpoint,
    // Never used once set but needs to stay alive.
    _callback: BoxedCallback<Box<FnMut(PacketListRef)>>,
}

/// A [MIDI object](https://developer.apple.com/reference/coremidi/midideviceref).
///
/// A MIDI device or external device, containing entities.
///
#[derive(Debug)]
#[derive(PartialEq)]
pub struct Device { object: Object }

mod coremidi_sys_ext;

mod object;
mod devices;
mod client;
mod ports;
mod packets;
mod properties;
mod endpoints;
mod notifications;
pub use devices::Devices;
pub use endpoints::destinations::Destinations;
pub use endpoints::sources::Sources;
pub use packets::{PacketBuffer, DynPacketBuffer, FixedPacketBuffer};
pub use packets::{PacketListRef, PacketListIterator, PacketRef};
pub use properties::{Properties, PropertyGetter, PropertySetter};
pub use notifications::Notification;

/// Unschedules previously-sent packets for all the endpoints.
/// See [MIDIFlushOutput](https://developer.apple.com/reference/coremidi/1495312-midiflushoutput).
///
pub fn flush() -> Result<(), OSStatus> {
    let status = unsafe { MIDIFlushOutput(0) };
    if status == 0 { Ok(()) } else { Err(status) }
}

/// Stops and restarts MIDI I/O.
/// See [MIDIRestart](https://developer.apple.com/reference/coremidi/1495146-midirestart).
///
pub fn restart() -> Result<(), OSStatus> {
    let status = unsafe { MIDIRestart() };
    if status == 0 { Ok(()) } else { Err(status) }
}
