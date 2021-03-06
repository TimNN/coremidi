extern crate coremidi;

use std::time::Duration;
use std::thread;

fn main() {
    let client = coremidi::Client::new("example-client").unwrap();
    let source = client.virtual_source("example-source").unwrap();

    let note_on = create_note_on(0, 64, 127);
    let note_off = create_note_off(0, 64, 127);

    for i in 0..10 {
        println!("[{}] Received note ...", i);

        source.received(note_on.as_ref()).unwrap();

        thread::sleep(Duration::from_millis(1000));

        source.received(note_off.as_ref()).unwrap();
    }
}

fn create_note_on(channel: u8, note: u8, velocity: u8) -> coremidi::DynPacketBuffer {
    let data = vec![
        0x90 | (channel & 0x0f),
        note & 0x7f,
        velocity & 0x7f];
    coremidi::PacketBuffer::from_data(0, data)
}

fn create_note_off(channel: u8, note: u8, velocity: u8) -> coremidi::DynPacketBuffer {
    let data = vec![
        0x80 | (channel & 0x0f),
        note & 0x7f,
        velocity & 0x7f];
    coremidi::PacketBuffer::from_data(0, data)
}
