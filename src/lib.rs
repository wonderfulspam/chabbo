use backends::Backend;
use markov::Chain;
use serde::Deserialize;
use tracing::debug;

use crate::backends::{deta::DetaService, ephemeral::EphemeralService, local::LocalService};

pub mod backends;

pub fn get_chain_from_text(text: &str) -> Chain<String> {
    debug!("loaded markov corpus of {} bytes", &text.len());
    let mut chain = Chain::of_order(2);
    for line in text.lines() {
        chain.feed_str(&line.to_lowercase());
    }
    debug!(
        "fed {} lines of text into markov chain",
        text.lines().count()
    );

    chain
}

#[derive(Deserialize)]
pub struct Config {
    // Will not be set in local env
    pub deta_project_key: Option<String>,
    // May not be set in local env but is always needed
    // Thus, we set a default
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "use_ephemeral")]
    pub use_ephemeral_backend: bool,
}

fn default_port() -> u16 {
    3000
}

fn use_ephemeral() -> bool {
    false
}

pub fn get_config_from_env() -> anyhow::Result<Config> {
    envy::from_env::<Config>().map_err(anyhow::Error::from)
}

pub fn choose_backend(config: &Config) -> Box<dyn Backend> {
    if let Some(deta_project_key) = &config.deta_project_key {
        debug!("creating Deta service");
        Box::new(DetaService::new(deta_project_key))
    } else if config.use_ephemeral_backend {
        debug!("using ephemeral in-memory backend");
        Box::<EphemeralService>::default()
    } else {
        debug!("using local backend");
        Box::<LocalService>::default()
    }
}
