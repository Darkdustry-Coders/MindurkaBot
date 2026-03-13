use humantime::Duration;
use poise::{Command, Context, command};
use tracing::info;

use crate::{
    discord::{DiscordData, DiscordError},
    surreal::{DB, fetch_profiles::fetch_profiles, types::ProfileType},
};

pub async fn commands() -> Vec<Command<DiscordData, DiscordError>> {
    vec![ban()]
}

#[command(slash_command, prefix_command)]
async fn ban(
    ctx: Context<'_, DiscordData, DiscordError>,
    _server: Option<String>,
    id: String,
    duration: Duration,
    #[rest] reason: String,
) -> Result<(), DiscordError> {
    let profile = fetch_profiles(
        ProfileType::Mindustry,
        id,
        crate::surreal::types::NeededProfiles {
            mindustry: true,
            discord: true,
            telegram: true,
        },
    )
    .await?;

    ctx.reply(format!("{:?}", profile)).await?;
    Ok(())
}
