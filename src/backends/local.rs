use super::{Database, FileStorage, Settings};
use anyhow::Result;
use directories::ProjectDirs;
use std::{
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
};
use tracing::debug;

#[derive(Clone)]
pub struct LocalService {
    proj_dirs: ProjectDirs,
}

impl LocalService {
    pub fn new() -> Self {
        let proj_dirs = ProjectDirs::from("org", "wonderfulspam", "Chabbo").unwrap();
        let service = Self { proj_dirs };
        // We're happy to fail fast, so we panic on any errors
        service.ensure_paths_exist().unwrap();
        service
    }

    fn ensure_paths_exist(&self) -> Result<(), std::io::Error> {
        for dir in [self.data_dir(), self.config_dir()] {
            debug!("creating {dir:?} if not exists");
            fs::create_dir_all(dir)?;
        }

        let db_file = self.db_file_path();
        debug!("creating {db_file:?} if not exists");
        OpenOptions::new().create(true).write(true).open(db_file)?;

        Ok(())
    }

    fn data_dir(&self) -> &Path {
        self.proj_dirs.data_dir()
    }

    fn config_dir(&self) -> &Path {
        self.proj_dirs.config_dir()
    }

    fn db_file_path(&self) -> PathBuf {
        self.config_dir().join("db.json")
    }

    fn get_fully_qualified_path(&self, path: &str) -> PathBuf {
        match path.starts_with('/') {
            true => path.into(),
            false => self.data_dir().join(path),
        }
    }
}

impl Default for LocalService {
    fn default() -> Self {
        Self::new()
    }
}

impl FileStorage for LocalService {
    fn list_files(&self) -> Result<Vec<String>> {
        let path = self.data_dir();
        debug!("scanning {path:?} for files");
        fs::read_dir(path)?
            .map(|entry| {
                entry
                    .map_err(anyhow::Error::from)
                    .map(|e| e.path().file_name().unwrap().to_string_lossy().to_string())
            })
            .collect()
    }

    fn upload_file(&self, name: &str, data: &[u8]) -> Result<String> {
        let data_dir = self.data_dir();
        let path = data_dir.join(name);
        debug!("writing file to {path:?}");
        fs::write(path, data)?;
        Ok(name.to_string())
    }

    fn get_file_contents(&self, path: &str) -> Result<String> {
        // Allow passing relative path
        let path = self.get_fully_qualified_path(path);
        debug!("loading markov corpus from {path:?}");
        fs::read_to_string(path).map_err(anyhow::Error::from)
    }
}

impl Database for LocalService {
    fn try_get_settings(&self) -> Option<Settings> {
        let path = self.db_file_path();
        debug!("loading settings from {path:?}");
        let settings_file_content = fs::read_to_string(path).ok()?;
        debug!("deserializing settings");
        serde_json::from_str(&settings_file_content).ok()
    }

    fn write_settings(&self, settings: &Settings) -> Result<()> {
        debug!("serializing settings");
        let json = serde_json::to_string(settings).unwrap();
        let path = self.db_file_path();
        debug!("writing settings to {path:?}");
        fs::write(path, json).map_err(anyhow::Error::from)
    }
}
