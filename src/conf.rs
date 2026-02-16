use crate::lib::conf::ConfigError::{ConfigStateCorrupt, InvalidEntryCount};
use std::fmt::Display;

use thiserror::Error;

#[derive(Debug)]
pub struct Config {
    pub entries: Vec<Entry>,
    lines: Vec<String>,
}

impl Config {
    const CONF_LOCATION: &'static str = "/etc/security/apple_watch.conf";

    pub fn load() -> Result<Self, ConfigError> {
        let raw_conf = std::fs::read_to_string(Self::CONF_LOCATION)?;
        let lines: Vec<String> = raw_conf.lines().map(ToString::to_string).collect();

        let entries: Vec<Entry> = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| !line.is_empty() && !line.starts_with('#'))
            .map(TryFrom::try_from)
            .collect::<Result<Vec<Entry>, ConfigError>>()?;

        Ok(Self { entries, lines })
    }

    pub fn get_user(&self, user: &String) -> Option<&Entry> {
        self.entries.iter().find(|entry| entry.user == *user)
    }

    pub fn update_user(&mut self, user: &String, encoded_irk: &String) -> bool {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.user == *user) {
            entry.encoded_irk.clone_from(encoded_irk);
            return true;
        }

        let line_number = self.lines.len();
        self.lines.push(String::new()); // Insert an empty line that the new entry will be placed at once saved
        self.entries.push(Entry {
            user: user.clone(),
            encoded_irk: encoded_irk.clone(),
            line_number,
        });

        false
    }

    pub fn save(&mut self) -> Result<(), ConfigError> {
        for (i, entry) in self.entries.iter().enumerate() {
            let Some(line) = self.lines.get_mut(entry.line_number) else {
                return Err(ConfigStateCorrupt(i, entry.line_number));
            };

            *line = entry.to_string();
        }

        let raw_config = self.lines.join("\n");
        Ok(std::fs::write(Self::CONF_LOCATION, raw_config)?)
    }
}

#[derive(Debug)]
pub struct Entry {
    pub user: String,
    pub encoded_irk: String,
    line_number: usize,
}

impl TryFrom<(usize, &'_ String)> for Entry {
    type Error = ConfigError;

    fn try_from(entry_line: (usize, &String)) -> Result<Self, Self::Error> {
        let (line_number, raw_entry) = entry_line;

        let values: Vec<&str> = raw_entry.split(';').collect();
        if values.len() < 2 {
            return Err(InvalidEntryCount(line_number, values.len()));
        }

        Ok(Self {
            user: values.first().expect("values length checked").to_string(),
            encoded_irk: values.get(1).expect("values length checked").to_string(),
            line_number,
        })
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{};{}", self.user, self.encoded_irk)
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Config entry (line {0}) has the wrong number ({1}) of entries")]
    InvalidEntryCount(usize, usize),

    #[error("Config entry {0} was expected to be on line {1} but it didn't exist")]
    ConfigStateCorrupt(usize, usize),
}
