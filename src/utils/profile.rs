use std::{collections::HashMap, fs, rc::Rc};

use serde::{Deserialize, Serialize};

use super::volume_control::VolumeControl;

#[derive(Debug)]
enum ConfigError {
    FaderNotFound(String),
    ButtonNotFound(String),
    GroupNotFound(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::FaderNotFound(fader) => write!(f, "Fader not found in config: {}", fader),
            ConfigError::ButtonNotFound(button) => {
                write!(f, "Fader not found in config: {}", button)
            }
            ConfigError::GroupNotFound(group) => {
                write!(f, "Group not found in config: {}", group)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
struct Button {
    control: u8,
    channel: u8,
    trigger: u8,
}

impl Button {
    pub fn triggered(&self, val: u8) -> bool {
        self.trigger == val
    }
}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
struct Fader {
    channel: u8,
    control: u8,
    min: u8,
    max: u8,
}

impl Fader {
    pub fn to_percentage(&self, val: u8) -> f64 {
        (val as f64 - self.min as f64) / (self.max as f64 - self.min as f64)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct ControlsConfig {
    buttons: HashMap<String, Button>,
    faders: HashMap<String, Fader>,
}

struct Controls {
    buttons: HashMap<String, Rc<Button>>,
    faders: HashMap<String, Rc<Fader>>,
}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
struct GroupConfig {
    volume_control: Vec<String>, // References to fader keys
    mute: Vec<String>,           // References to button keys
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct Group {
    name: String,
    volume_control: Vec<Rc<Fader>>,
    mute: Vec<Rc<Button>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct ProfileConfig {
    controls: ControlsConfig, // Include Controls to ensure buttons and faders are defined
    groups: HashMap<String, GroupConfig>,
    mapping: HashMap<String, String>, // Mapping of Group to application
}
pub struct Profile {
    controls: Controls,
    mapping: HashMap<Group, String>,
}

impl Profile {
    fn get_faders(
        group: &GroupConfig,
        faders: &HashMap<String, Rc<Fader>>,
    ) -> Result<Vec<Rc<Fader>>, ConfigError> {
        if !group.volume_control.is_empty() {
            group
                .volume_control
                .iter()
                .map(|fader| {
                    faders
                        .get(fader)
                        .map(|v| Rc::clone(v)) // Clone the Rc<Fader> only once
                        .ok_or_else(|| ConfigError::FaderNotFound(fader.clone()))
                })
                .collect::<Result<Vec<Rc<Fader>>, ConfigError>>() // Collects Vec<Rc<Fader>>
        } else {
            Ok(Vec::new()) // Return an empty Vec if volume_control is empty
        }
    }

    fn get_buttons(
        group: &GroupConfig,
        buttons: &HashMap<String, Rc<Button>>,
    ) -> Result<Vec<Rc<Button>>, ConfigError> {
        if !group.volume_control.is_empty() {
            group
                .volume_control
                .iter()
                .map(|fader| {
                    buttons
                        .get(fader)
                        .map(|v| Rc::clone(v)) // Clone the Rc<Fader> only once
                        .ok_or_else(|| ConfigError::FaderNotFound(fader.clone()))
                })
                .collect::<Result<Vec<Rc<Button>>, ConfigError>>() // Collects Vec<Rc<Fader>>
        } else {
            Ok(Vec::new()) // Return an empty Vec if volume_control is empty
        }
    }

    pub fn new(config: &ProfileConfig) -> Result<Profile, ConfigError> {
        let buttons: HashMap<String, Rc<Button>> = config
            .controls
            .buttons
            .iter()
            .map(|(key, button)| (key.clone(), Rc::new(button.clone())))
            .collect();

        let faders: HashMap<String, Rc<Fader>> = config
            .controls
            .faders
            .iter()
            .map(|(key, fader)| (key.clone(), Rc::new(fader.clone())))
            .collect();

        let groups: Vec<Group> = config
            .groups
            .iter()
            .map(|group| {
                let faders =
                    Profile::get_faders(group.1, &faders).map_err(|e| ConfigError::from(e))?;

                let buttons =
                    Profile::get_buttons(group.1, &buttons).map_err(|e| ConfigError::from(e))?;

                Ok(Group {
                    name: group.0.to_owned(),
                    volume_control: faders,
                    mute: buttons,
                })
            })
            .collect::<Result<Vec<Group>, ConfigError>>()?;

        let mapping = config
            .mapping
            .iter()
            .map(|map| {
                // Search for the group in the groups vector
                if let Some(group) = groups.iter().find(|g| g.name == *map.0) {
                    Ok((group.clone(), map.1.to_owned())) // Clone the group since we're borrowing
                } else {
                    Err(ConfigError::GroupNotFound(map.0.to_owned()))
                }
            })
            .collect::<Result<HashMap<Group, String>, ConfigError>>()?;

        Ok(Profile {
            controls: Controls { buttons, faders },
            mapping,
        })
    }

    //Returns fader + application name/ output description, None if there is no application
    pub fn get_volume_control(&self, channel: u8, control: u8) -> Option<(String, Rc<Fader>)> {
        for map in &self.mapping {
            if let Some(fader) = map
                .0
                .volume_control
                .iter()
                .find(|&f| f.channel == channel && f.control == control)
            {
                if !map.1.is_empty() {
                    return Some((map.1.clone(), Rc::clone(fader)));
                }
            }
        }

        None
    }

    //Returns button + application name/ output description, None if there is no application
    pub fn get_mute(&self, channel: u8, control: u8) -> Option<(String, Rc<Button>)> {
        for map in &self.mapping {
            if let Some(button) = map
                .0
                .mute
                .iter()
                .find(|&f| f.channel == channel && f.control == control)
            {
                if !map.1.is_empty() {
                    return Some((map.1.clone(), Rc::clone(button)));
                }
            }
        }

        None
    }
}
