use std::{
    cell::RefCell,
    fs,
    io::{stdin, stdout, Write},
    rc::Rc,
};

use midir::{Ignore, MidiInput, MidiInputPort};
use pulsectl::{
    controllers::{AppControl, DeviceControl, SinkController},
    ControllerError,
};
use utils::volume_control::{self, Application, OutputDevice, VolumeControl};

use directories::ProjectDirs;

use serde::{Deserialize, Serialize};

mod utils;

#[derive(Deserialize, Debug, Serialize, Clone)]
struct Config {
    midi_mixer: MidiMixerConfig,
}

#[derive(Deserialize, Debug, Serialize, Clone, Default)]
struct MidiMixerConfig {
    port_name: String,
    sliders: Vec<SliderConfig>,
    buttons: Vec<ButtonConfig>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
struct SliderConfig {
    slider_id: u32,
    application: String,
    min_value: u8,
    max_value: u8,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
enum ButtonFunction {
    Mute,
    UnMute,
    Play,
    Stop,
    SkipForward,
    SkipBackward,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
struct ButtonConfig {
    button_id: u32,
    function: ButtonFunction,
    pressed_value: u8,
    release_value: u8,
}

pub struct MidiMixer {
    config: Rc<RefCell<MidiMixerConfig>>,
    devices: Vec<OutputDevice>,
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

    pub fn fetch_applications(&mut self) -> Result<(), ControllerError> {
        let applications = self.handler.borrow_mut().list_applications()?;
        self.applications = applications
            .iter()
            .map(|app| {
                Application::new(
                    app.index,
                    match app.proplist.get("application.process.binary") {
                        Some(binary_name) => std::str::from_utf8(binary_name).unwrap().to_string(),
                        None => "".to_string(),
                    },
                    Rc::clone(&self.handler),
                )
            })
            .collect();

        Ok(())
    }

    pub fn get_applications(&self) -> &Vec<Application> {
        &self.applications
    }

    pub fn fetch_playback_devices(&mut self) -> Result<(), ControllerError> {
        let devices = self.handler.borrow_mut().list_devices()?;
        self.devices = devices
            .iter()
            .map(|device| {
                OutputDevice::new(
                    device.index,
                    device.description.clone().unwrap_or("".to_string()),
                    Rc::clone(&self.handler),
                )
            })
            .collect();
        Ok(())
    }

    pub fn get_playback_device(&self) -> &Vec<OutputDevice> {
        &self.devices
    }
}

fn main() {
    if let Some(proj_dirs) = ProjectDirs::from("io", "CrazyPandabier", "midi-mixer") {
        let config_dir = proj_dirs.config_dir();
        let config_path = config_dir.join("config.toml");
        let config_file = fs::read_to_string(config_path.clone());

        let config = match config_file {
            Ok(file) => toml::from_str(&file).unwrap(),
            Err(_) => Config {
                midi_mixer: MidiMixerConfig {
                    port_name: "nanoKONTROL2:nanoKONTROL2 nanoKONTROL2 _ CTR 28:0".to_string(),
                    sliders: Vec::new(),
                    buttons: Vec::new(),
                },
            },
        };
        let toml_str = toml::to_string_pretty(&config).unwrap();
        println!("{}", config_path.to_str().unwrap());
        fs::write(config_path, toml_str).unwrap();

        let mut input = String::new();

        let mut midi_in = MidiInput::new("midir reading input").unwrap();
        midi_in.ignore(Ignore::None);

        // Get an input port (read from console if multiple are available)
        let in_ports = midi_in.ports();

        let in_port: MidiInputPort = in_ports
            .iter()
            .find_map(|port| {
                if midi_in
                    .port_name(port)
                    .unwrap()
                    .contains(&config.midi_mixer.port_name)
                {
                    Some(port.clone()) // Assuming MidiInputPort is `Copy`. Otherwise, use port.clone()
                } else {
                    None
                }
            })
            .expect("No matching MIDI input port found");

        println!("\nOpening connection");
        let in_port_name = midi_in.port_name(&in_port).unwrap();

        // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
        let _conn_in = midi_in.connect(
            &in_port,
            "midir-read-input",
            move |stamp, message, _| {
                println!("{}: {:?} (len = {})", stamp, message, message.len());
            },
            (),
        );

        println!(
            "Connection open, reading input from '{}' (press enter to exit) ...",
            in_port_name
        );

        let mut mixer = MidiMixer::new(Rc::new(RefCell::new(config.midi_mixer.clone())));

        mixer.fetch_applications().unwrap();

        let applications = mixer.get_applications();

        loop {
            for app in applications {
                app.set_volume(100.0);
            }
        }
    }
}
