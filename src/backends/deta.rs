use anyhow::Result;
use deta_rs::{utils, Deta};
use serde::Deserialize;
use tracing::debug;

use super::{Backend, Database, FileStorage, Settings};

const DRIVE_NAME: &str = "corpus";
const BASE_NAME: &str = "settings";

#[derive(Clone)]
pub struct DetaService {
    client: Deta,
}

#[derive(Deserialize)]
struct FileList {
    names: Vec<String>,
}

#[derive(Deserialize)]
struct File {
    name: String,
}

impl FileStorage for DetaService {
    fn list_files(&self) -> Result<Vec<String>> {
        let drive = self.client.drive(DRIVE_NAME);

        debug!("querying deta drive for files");
        let filenames: FileList = serde_json::from_value(drive.list(None, None, None)?)
            .expect("Response must contain 'names' field");
        let filenames = filenames.names;
        debug!("found {} files", filenames.len());
        Ok(filenames)
    }

    fn upload_file(&self, name: &str, data: &[u8]) -> Result<String> {
        let drive = self.client.drive(DRIVE_NAME);
        debug!("uploading {name} to Drive");
        let file: File = serde_json::from_value(drive.put(name, data)?)
            .expect("Response must contain 'name' field");
        Ok(file.name)
    }

    fn get_file_contents(&self, name: &str) -> Result<String> {
        let drive = self.client.drive(DRIVE_NAME);
        debug!("loading contents of {name} from Drive");
        drive.get(name).map_err(anyhow::Error::from)
    }
}

impl Database for DetaService {
    fn try_get_settings(&self) -> Option<Settings> {
        let base = self.client.base(BASE_NAME);

        debug!("getting settings from Base");
        let res = base.get("settings").ok()?;
        serde_json::from_value(res).ok()
    }

    fn write_settings(&self, settings: &Settings) -> Result<()> {
        let base = self.client.base(BASE_NAME);
        let record = utils::Record {
            key: Some("settings".to_string()),
            value: Some(serde_json::to_value(settings).unwrap()),
            expires_in: None,
            expires_at: None,
        };
        base.put(vec![record])?;
        Ok(())
    }
}

impl Backend for DetaService {}

impl DetaService {
    pub fn new(deta_project_key: String) -> Self {
        Self {
            client: Deta::new(&deta_project_key),
        }
    }
}
