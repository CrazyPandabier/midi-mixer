use std::fs;

use utils::{
    midi_mixer::{self, MidiMixer},
    profile::{Profile, ProfileConfig},
};

mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let toml = fs::read_to_string("config.toml")?;
    let config: ProfileConfig = toml::from_str(&toml)?;
    let profile = Profile::new(&config)?;

    let mut midi_mixer = MidiMixer::new(Profile::new(&config)?).unwrap();

    loop {
        midi_mixer.update()?;
    }
}
