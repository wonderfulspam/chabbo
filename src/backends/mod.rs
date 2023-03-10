pub mod deta;
pub mod ephemeral;
pub mod local;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tracing::warn;

const DEFAULT_MARKOV_CORPUS: &str = include_str!("../../dorian_gray.txt");

pub trait FileStorage {
    fn list_files(&self) -> Result<Vec<String>>;
    fn upload_file(&self, name: &str, data: &[u8]) -> Result<String>;
    fn get_file_contents(&self, name: &str) -> Result<String>;
}

pub trait Database {
    fn get_settings(&self) -> Result<Settings> {
        match self.try_get_settings() {
            Some(settings) => Ok(settings),
            None => {
                warn!("settings could not be deserialized");
                let settings = Settings::default();
                self.write_settings(&settings)?;
                Ok(settings)
            }
        }
    }

    fn try_get_settings(&self) -> Option<Settings>;

    fn write_settings(&self, settings: &Settings) -> Result<()>;

    fn get_active_corpus_name(&self) -> Result<String> {
        let settings = self.get_settings()?;
        Ok(settings.active_corpus.to_string())
    }

    fn set_active_corpus_name(&self, name: &str) -> Result<String> {
        let mut settings = self.get_settings()?;
        settings.active_corpus = name.into();
        self.write_settings(&settings)?;
        Ok(settings.active_corpus.to_string())
    }
}

pub trait Backend: FileStorage + Database + Send + Sync + 'static {
    fn get_initial_corpus(&self) -> Result<String>;
}

impl<T> Backend for T
where
    T: FileStorage + Database + Send + Sync + 'static,
{
    fn get_initial_corpus(&self) -> Result<String> {
        match self.get_settings()?.active_corpus {
            CorpusType::Default => Ok(DEFAULT_MARKOV_CORPUS.to_string()),
            CorpusType::FromFile { path } => self.get_file_contents(&path),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
enum CorpusType {
    Default,
    FromFile { path: String },
}

impl Display for CorpusType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_name = match self {
            CorpusType::Default => "Default",
            CorpusType::FromFile { path } => path,
        };
        write!(f, "{display_name}")
    }
}

impl From<&str> for CorpusType {
    fn from(value: &str) -> Self {
        match value {
            "Default" => CorpusType::Default,
            path => CorpusType::FromFile {
                path: path.to_string(),
            },
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Settings {
    active_corpus: CorpusType,
}

impl Settings {
    fn new() -> Self {
        Self {
            active_corpus: CorpusType::Default,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}
