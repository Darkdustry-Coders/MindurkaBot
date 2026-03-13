use std::{path::PathBuf, process::exit};

use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;
use tracing::error;
use url::Url;

use crate::{args::get_app_args, discord::config::DiscordConfig};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub shared_config_path: PathBuf,
    #[cfg(feature = "discord")]
    pub discord: DiscordConfig,
    pub services: Vec<String>,
}

static CONFIG_SINGLETON: OnceCell<Config> = OnceCell::const_new();

pub async fn get_config() -> &'static Config {
    CONFIG_SINGLETON
        .get_or_init(|| async {
            let path = match &get_app_args().config {
                Some(path) => path,
                None => &{
                    let mut root = std::env::current_dir().expect("А где файловая система?");
                    root.push("botConfig.toml");
                    root
                },
            };
            let config_str = match tokio::fs::read_to_string(path).await {
                Ok(config_str) => config_str,
                Err(err) => {
                    error!("Failed to read config: {:?}", err);
                    exit(1);
                    #[allow(unreachable_code)]
                    {
                        unreachable!("Посхалко")
                    }
                }
            };

            match toml::from_str(&config_str) {
                Ok(config) => config,
                Err(err) => {
                    error!("Failed to parse config: {:?}", err);
                    exit(1);
                }
            }
        })
        .await
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedConfig {
    pub rabbit_mq_url: Url,
    pub surreal_db_url: Url,
}

static SHAREDCONFIG_SINGLETON: OnceCell<SharedConfig> = OnceCell::const_new();

pub async fn get_shared_config() -> &'static SharedConfig {
    SHAREDCONFIG_SINGLETON
        .get_or_init(|| async {
            let path = &get_config().await.shared_config_path;
            let config_str = match tokio::fs::read_to_string(path).await {
                Ok(config_str) => config_str,
                Err(err) => {
                    error!("Failed to read shared config: {:?}", err);
                    exit(1);
                }
            };
            match toml::from_str(&config_str) {
                Ok(config) => config,
                Err(err) => {
                    error!("Failed to parse config: {:?}", err);
                    exit(1);
                }
            }
        })
        .await
}
