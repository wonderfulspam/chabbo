use std::collections::HashMap;

use super::{Database, FileStorage, Settings};
use once_cell::sync::Lazy;
use std::sync::Mutex;

static FILES: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static SETTINGS: Lazy<Mutex<Option<Settings>>> = Lazy::new(|| Mutex::new(None));
pub struct EphemeralService;

impl EphemeralService {
    fn new() -> Self {
        Self
    }
}

impl Default for EphemeralService {
    fn default() -> Self {
        Self::new()
    }
}

impl FileStorage for EphemeralService {
    fn list_files(&self) -> anyhow::Result<Vec<String>> {
        Ok(FILES
            .lock()
            .unwrap()
            .keys()
            .map(|s| s.to_string())
            .collect())
    }

    fn upload_file(&self, name: &str, data: &[u8]) -> anyhow::Result<String> {
        FILES
            .lock()
            .unwrap()
            .insert(name.to_string(), String::from_utf8_lossy(data).into_owned());
        Ok(name.to_string())
    }

    fn get_file_contents(&self, name: &str) -> anyhow::Result<String> {
        FILES
            .lock()
            .unwrap()
            .get(name)
            .map(|s| s.to_owned())
            .ok_or(anyhow::anyhow!("File not found"))
    }
}
impl Database for EphemeralService {
    fn try_get_settings(&self) -> Option<Settings> {
        SETTINGS.lock().unwrap().as_ref().cloned()
    }

    fn write_settings(&self, settings: &Settings) -> anyhow::Result<()> {
        let mut existing_settings = SETTINGS.lock().unwrap();
        *existing_settings = Some(settings.clone());
        Ok(())
    }
}
