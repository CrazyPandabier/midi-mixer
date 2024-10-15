use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct MidiMessage {
    pub channel: u8,
    pub control: u8,
    pub value: u8,
}

impl MidiMessage {
    fn new(message: &[u8]) -> Option<MidiMessage> {
        if message.len() < 3 {
            return None; // Not a complete MIDI message
        }

        let status = message[0];
        let channel = status & 0x0F; // Get the channel (lower 4 bits)
        let message_type = status & 0xF0; // Get the message type

        // Check if this is a Control Change message (0xB0 to 0xBF)
        if message_type == 0xB0 {
            let control = message[1];
            let value = message[2];

            Some(MidiMessage {
                channel,
                control,
                value,
            })
        } else {
            None // Not a Control Change message
        }
    }
}

pub trait MidiCallback: Send + 'static {
    fn handle_midi_message(&mut self, message: MidiMessage);
}

pub struct MidiController<T: MidiCallback> {
    input_connection: Option<MidiInputConnection<()>>,
    callback_handler: Arc<Mutex<T>>,
}

impl<T: MidiCallback> MidiController<T> {
    pub fn new(callback_handler: Arc<Mutex<T>>) -> MidiController<T> {
        MidiController {
            input_connection: None,
            callback_handler,
        }
    }

    pub fn connect_input(&mut self, port_name: &str) -> Result<(), Box<dyn Error>> {
        let mut midi_in = MidiInput::new("MidiController Input")?;
        midi_in.ignore(Ignore::None); // To avoid ignoring MIDI events

        let in_ports = midi_in.ports();
        let in_port = in_ports
            .iter()
            .find(|p| midi_in.port_name(p).map_or(false, |name| name == port_name))
            .ok_or_else(|| format!("No input port found with name: {}", port_name))?;

        // Clone the Arc for use in the closure
        let callback_handler = Arc::clone(&self.callback_handler);

        let conn_in = midi_in.connect(
            in_port,
            "Midi Input Connection",
            move |_, message, _| {
                if let Some(msg) = MidiMessage::new(message) {
                    callback_handler.lock().unwrap().handle_midi_message(msg);
                }
            },
            (),
        )?;

        self.input_connection = Some(conn_in);

        Ok(())
    }
}
