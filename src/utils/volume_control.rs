use std::cell::Ref;
use std::{cell::RefCell, rc::Rc};

use pulsectl::controllers::AppControl;
use pulsectl::controllers::DeviceControl;
use pulsectl::controllers::SinkController;
use pulsectl::ControllerError;

use crate::MidiMixerConfig;

pub trait VolumeControl {
    fn set_volume(&self, val: f64) -> Result<(), ControllerError>;
    fn mute(&self) -> Result<(), ControllerError>;
    fn unmute(&self) -> Result<(), ControllerError>;
    fn get_name(&self) -> &str;
}

pub struct OutputDevice {
    index: u32,
    description: String,
    handler: Rc<RefCell<SinkController>>,
}

impl OutputDevice {
    pub fn new(
        index: u32,
        description: String,
        handler: Rc<RefCell<SinkController>>,
    ) -> OutputDevice {
        OutputDevice {
            index,
            description,
            handler,
        }
    }
}

impl VolumeControl for OutputDevice {
    fn get_name(&self) -> &str {
        &self.description
    }

    fn set_volume(&self, val: f64) -> Result<(), ControllerError> {
        let mut handler = self.handler.borrow_mut();
        let current_volume: u8 = handler
            .get_device_by_index(self.index)?
            .volume
            .avg()
            .print()
            .trim_end_matches('%')
            .trim_ascii()
            .parse()
            .expect("Failed to convert volume to integer");

        let delta = val - (current_volume as f64 / 100.0);

        println!("delta {}", delta);

        if delta < 0.0 {
            handler.decrease_device_volume_by_percent(self.index, delta.abs());
        } else {
            handler.increase_device_volume_by_percent(self.index, delta);
        }

        Ok(())
    }

    fn mute(&self) -> Result<(), ControllerError> {
        self.handler
            .borrow_mut()
            .set_device_mute_by_index(self.index, true);
        Ok(())
    }

    fn unmute(&self) -> Result<(), ControllerError> {
        self.handler
            .borrow_mut()
            .set_device_mute_by_index(self.index, false);
        Ok(())
    }
}

pub struct Application {
    index: u32,
    name: String,
    handler: Rc<RefCell<SinkController>>,
}

impl Application {
    pub fn new(index: u32, name: String, handler: Rc<RefCell<SinkController>>) -> Application {
        Application {
            index,
            name,
            handler,
        }
    }
}

impl VolumeControl for Application {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn set_volume(&self, val: f64) -> Result<(), ControllerError> {
        let mut handler = self.handler.borrow_mut();
        let current_volume: u8 = handler
            .get_app_by_index(self.index)?
            .volume
            .avg()
            .print()
            .trim_end_matches('%')
            .trim_ascii()
            .parse()
            .expect("Failed to convert volume to integer");

        let delta = val - (current_volume as f64 / 100.0);

        println!("delta {}", delta);

        if delta < 0.0 {
            handler.decrease_app_volume_by_percent(self.index, delta.abs());
        } else {
            handler.increase_app_volume_by_percent(self.index, delta);
        }

        Ok(())
    }

    fn mute(&self) -> Result<(), ControllerError> {
        self.handler.borrow_mut().set_app_mute(self.index, true)?;
        Ok(())
    }

    fn unmute(&self) -> Result<(), ControllerError> {
        self.handler.borrow_mut().set_app_mute(self.index, false)?;
        Ok(())
    }
}
