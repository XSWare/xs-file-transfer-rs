use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    sync::Mutex,
};

/// generic persistent key/value settings store.
///
/// values are persisted to a single file in the user's home directory so
/// more settings can be added later on without changing the storage format.
pub struct Settings {
    file_path: Option<PathBuf>,
    values: Mutex<HashMap<String, String>>,
}

const SETTINGS_FILE_NAME: &str = ".xs_file_transfer_settings";

impl Settings {
    /// loads the settings file from the user's home directory, if present.
    pub fn load() -> Self {
        let file_path = get_settings_file_path();
        let values = file_path
            .as_ref()
            .and_then(|path| fs::read_to_string(path).ok())
            .map(|content| parse(&content))
            .unwrap_or_default();

        Self {
            file_path,
            values: Mutex::new(values),
        }
    }

    /// returns the value of `key` if it was previously persisted.
    pub fn get(&self, key: &str) -> Option<String> {
        self.values.lock().unwrap().get(key).cloned()
    }

    /// sets `key` to `value` and persists the whole settings store to disk.
    pub fn set(&self, key: &str, value: impl Into<String>) {
        let mut values = self.values.lock().unwrap();
        values.insert(key.to_string(), value.into());
        self.persist(&values);
    }

    fn persist(&self, values: &HashMap<String, String>) {
        let Some(path) = &self.file_path else {
            return;
        };

        let mut file = File::create(path).unwrap();
        file.write_all(&serialize(values).into_bytes()).unwrap();
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::load()
    }
}

fn get_settings_file_path() -> Option<PathBuf> {
    directories::UserDirs::new().map(|user_dirs| user_dirs.home_dir().join(SETTINGS_FILE_NAME))
}

fn parse(content: &str) -> HashMap<String, String> {
    content
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn serialize(values: &HashMap<String, String>) -> String {
    values
        .iter()
        .map(|(key, value)| format!("{}={}", key, value))
        .collect::<Vec<_>>()
        .join("\n")
}
