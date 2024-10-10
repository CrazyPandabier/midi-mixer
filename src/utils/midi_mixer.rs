use std::cell::Ref;
use std::{cell::RefCell, rc::Rc};

use pulsectl::controllers::AppControl;
use pulsectl::controllers::DeviceControl;
use pulsectl::controllers::SinkController;
use pulsectl::ControllerError;

use crate::MidiMixerConfig;

pub struct Device {
    index: u32,
    description: String,
    volume: u8,
    handler: Rc<RefCell<SinkController>>,
}

impl Device {
    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn set_volume_midi(&self, val: u8) {
        todo!()
    }

    pub fn mute(&self) {
        todo!()
    }

    pub fn unmute(&self) {
        todo!()
    }
}

pub struct Application {
    index: u32,
    name: String,
    volume: u8,
    handler: Rc<RefCell<SinkController>>,
}

impl Application {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_volume_midi(&self, val: u8) {
        todo!()
    }

    pub fn mute(&self) {
        todo!()
    }

    pub fn unmute(&self) {
        todo!()
    }
}

pub struct MidiMixer {
    config: Rc<RefCell<MidiMixerConfig>>,
    devices: Vec<Device>,
    applications: Vec<Application>,
    handler: Rc<RefCell<SinkController>>,
}

impl MidiMixer {
    pub fn new(config: Rc<RefCell<MidiMixerConfig>>) -> MidiMixer {
        MidiMixer {
            config,
            devices: Vec::new(),
            applications: Vec::new(),
            handler: Rc::new(RefCell::new(SinkController::create().unwrap())),
        }
    }

    pub fn get_applications(&self) -> Result<&Vec<Application>, ControllerError> {
        if self.applications.is_empty() {
            let applications = self.handler.borrow_mut().list_applications()?;
        }

        Ok(&self.applications)
    }

    pub fn get_playback_device(&self) -> Result<&Vec<Device>, ControllerError> {
        if self.devices.is_empty() {
            let devices = self.handler.borrow_mut().list_devices()?;
        }

        Ok(&self.devices)
    }
}
