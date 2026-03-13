use std::time::{Instant, SystemTime, UNIX_EPOCH};

use poise::serenity_prelude::{
    ChannelId, ChannelType, Context, CreateEmbed, EditThread, Embed, FormattedTimestamp,
    FormattedTimestampStyle, ForumTagId, GuildChannel, Timestamp,
};
use tracing::{error, info};

use crate::{config::get_config, discord::reports::parser::multi_parser};

pub mod parser;

pub async fn main_report_handler(ctx: Context, mut thread: GuildChannel) {
    if thread.kind != ChannelType::PublicThread {
        return;
    }

    if thread.parent_id != Some(ChannelId::new(get_config().await.discord.reports.forum_id)) {
        return;
    }

    let mut tags = thread.applied_tags.clone();

    tags.push(ForumTagId::new(
        get_config().await.discord.reports.awaiting_tag_id,
    ));

    let _ = thread
        .edit_thread(&ctx, EditThread::new().applied_tags(tags))
        .await;

    let initial_msg = match ctx
        .http
        .get_message(thread.id, Into::<u64>::into(thread.id).into())
        .await
    {
        Ok(msg) => msg,
        Err(err) => {
            error!("Failed to get initial message: {:?}", err);
            return;
        }
    };

    let content = &initial_msg.content;

    let parsed_report = multi_parser(content);
    let rule = if let Some(rule) = parsed_report.rule {
        get_config().await.discord.reports.rules.get(&rule)
    } else {
        None
    };

    let ban_end_time = rule
        .iter()
        .map(|it| &it.duration)
        .map(|it| SystemTime::now() + **it)
        .map(|it| {
            FormattedTimestamp::new(
                Timestamp::from_unix_timestamp(
                    it.duration_since(UNIX_EPOCH).unwrap().as_secs_f32() as i64,
                )
                .unwrap(),
                Some(FormattedTimestampStyle::ShortDateTime),
            )
        })
        .map(|it| it.to_string())
        .next();

    let embed = CreateEmbed::new()
        .title("Report")
        .field(
            "Player:",
            parsed_report
                .id
                .unwrap_or("Could not get from report".into()),
            false,
        )
        .field(
            "Violated rule:",
            parsed_report
                .rule
                .map(|it| it.to_string())
                .unwrap_or("Could not get from report".into()),
            false,
        )
        .field(
            "Ban until:",
            ban_end_time.unwrap_or("Could not get from report".into()),
            false,
        )
        .field(
            "Reason:",
            parsed_report
                .reason
                .unwrap_or("Could not get from report".into()),
            false,
        );
}
