use std::{
    cell::RefCell,
    error::Error,
    rc::Rc,
    sync::{Arc, Mutex},
};

use pulsectl::{
    controllers::{AppControl, DeviceControl, SinkController},
    ControllerError,
};

use super::{
    midi_controller::{MidiCallback, MidiController, MidiMessage},
    profile::Profile,
    volume_control::{self, Application, OutputDevice, VolumeControl},
};

struct MidiHandler {
    previous_message: MidiMessage,
    last_message: MidiMessage,
}

impl MidiHandler {
    pub fn new() -> MidiHandler {
        MidiHandler {
            last_message: MidiMessage {
                channel: 0,
                control: 0,
                value: 0,
            },
            previous_message: MidiMessage {
                channel: 0,
                control: 0,
                value: 0,
            },
        }
    }

    pub fn get_last_message(&mut self) -> MidiMessage {
        self.previous_message = self.last_message;
        self.last_message
    }

    pub fn is_available(&self) -> bool {
        if self.previous_message != self.last_message {
            return true;
        }

        false
    }
}

impl MidiCallback for MidiHandler {
    fn handle_midi_message(&mut self, message: MidiMessage) {
        self.last_message = message;
    }
}

pub struct MidiMixer {
    midi_handler: Arc<Mutex<MidiHandler>>,
    controller: MidiController<MidiHandler>,
    profile: Profile,
    mixer_handler: Rc<RefCell<SinkController>>,
}

impl MidiMixer {
    pub fn new(profile: Profile) -> Result<MidiMixer, Box<dyn Error>> {
        let midi_handler = Arc::new(Mutex::new(MidiHandler::new()));
        let mut controller = MidiController::new(Arc::clone(&midi_handler));
        controller.connect_input(&profile.get_midi_controller_name())?;
        Ok(MidiMixer {
            midi_handler: Arc::clone(&midi_handler),
            controller,
            profile,
            mixer_handler: Rc::new(RefCell::new(SinkController::create().unwrap())),
        })
    }

    fn get_applications(&mut self) -> Result<Vec<Application>, ControllerError> {
        let applications = self.mixer_handler.borrow_mut().list_applications()?;
        Ok(applications
            .iter()
            .map(|app| {
                Application::new(
                    app.index,
                    match app.proplist.get("application.process.binary") {
                        Some(binary_name) => std::str::from_utf8(binary_name).unwrap().to_string(),
                        None => "".to_string(),
                    },
                    Rc::clone(&self.mixer_handler),
                )
            })
            .collect())
    }

    fn get_playback_devices(&mut self) -> Result<Vec<OutputDevice>, ControllerError> {
        let devices = self.mixer_handler.borrow_mut().list_devices()?;
        Ok(devices
            .iter()
            .map(|device| {
                OutputDevice::new(
                    device.index,
                    device.description.clone().unwrap_or("".to_string()),
                    Rc::clone(&self.mixer_handler),
                )
            })
            .collect())
    }

    fn get_volume_control(
        &mut self,
        sink_name: String,
    ) -> Result<Option<Box<dyn VolumeControl>>, ControllerError> {
        let applications = self.get_applications()?;
        for app in applications {
            println!("{}", app.get_name());
            if sink_name.trim().to_ascii_lowercase()
                == app
                    .get_name()
                    .trim()
                    .to_ascii_lowercase()
                    .trim_end_matches('\0')
            {
                return Ok(Some(Box::new(app)));
            }
        }

        let devices = self.get_playback_devices()?;
        for device in devices {
            if device.get_name() == sink_name {
                return Ok(Some(Box::new(device)));
            }
        }

        Ok(None)
    }

    pub fn update(&mut self) -> Result<(), ControllerError> {
        let mut handler = self.midi_handler.lock().unwrap();

        if handler.is_available() {
            let message = handler.get_last_message();
            drop(handler);

            if let Some((sink_name, button)) =
                self.profile.get_mute(message.channel, message.control)
            {
                if button.triggered(message.value) {
                    if let Some(volume_control) = self.get_volume_control(sink_name)? {
                        volume_control.toggle_mute()?;
                    }
                }
            }

            if let Some((sink_name, fader)) = self
                .profile
                .get_volume_control(message.channel, message.control)
            {
                let percent = fader.to_percentage(message.value);

                if let Some(volume_control) = self.get_volume_control(sink_name)? {
                    volume_control.set_volume(percent)?;
                }
            }
        }

        Ok(())
    }
}
