use std::{collections::HashMap, error::Error, fmt::Display, str::FromStr};

use humantime::Duration;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_tuple::{Deserialize_tuple, Serialize_tuple};
use serde_with::{DisplayFromStr, serde_as};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub token: String,
    pub guild_id: u64,
    pub routes: HashMap<String, RouteEntry>,
    pub reports: ReportsConfig,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RouteEntry {
    pub webhook_url: String,
    pub channel_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportsConfig {
    pub forum_id: u64,
    pub awaiting_tag_id: u64,
    pub resolved_tag_id: u64,
    pub denied_tag_id: u64,
    pub rules: HashMap<Decimal, RuleEntry>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize_tuple, Deserialize_tuple)]
pub struct RuleEntry {
    #[serde_as(as = "DisplayFromStr")]
    pub punishment_type: PunishmentType,
    #[serde_as(as = "DisplayFromStr")]
    pub duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PunishmentType {
    Ban,
    Kick,
    Mute,
}

impl Display for PunishmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ban => write!(f, "ban"),
            Self::Kick => write!(f, "kick"),
            Self::Mute => write!(f, "mute"),
        }
    }
}

impl FromStr for PunishmentType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "ban" => Ok(Self::Ban),
            "kick" => Ok(Self::Kick),
            "mute" => Ok(Self::Mute),
            _ => Err(anyhow::Error::msg("Wrong punshiment type")),
        }
    }
}
