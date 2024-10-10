use std::fs;

use pulsectl::controllers::AppControl;
use pulsectl::controllers::DeviceControl;
use pulsectl::controllers::SinkController;

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

fn main() {
    if let Some(proj_dirs) = ProjectDirs::from("io", "CrazyPandabier", "midi-mixer") {
        let config_dir = proj_dirs.config_dir();
        let config_path = config_dir.join("config.toml");
        let config_file = fs::read_to_string(config_path.clone());

        let mut config = match config_file {
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
        fs::write(config_path, toml_str).unwrap();
    }

    // create handler that calls functions on playback devices and apps
    let mut handler = SinkController::create().unwrap();

    let devices = handler
        .list_devices()
        .expect("Could not get list of playback devices.");

    let applications = handler
        .list_applications()
        .expect("Could not get list of applications");

    println!("Playback Devices: ");
    for dev in devices.clone() {
        println!(
            "[{}] {}, Volume: {}",
            dev.index,
            dev.description.as_ref().unwrap(),
            dev.volume.print()
            handler.increase_app_volume_by_percent(index, delta);
        );
    }

    for app in applications.clone() {
        println!(
            "[{}] {}, Volume: {}",
            app.index,
            {
                match app.proplist.get("application.process.binary") {
                    Some(binary_name) => std::str::from_utf8(binary_name).unwrap(),
                    None => "",
                }
            },
            app.volume.print()
        )
    }
}
