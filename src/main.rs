use std::{any::Any, time::Duration};

use mindurka_rabbitmq_rust::{InjectQueues, QueueWithHandles, Rabbitmq};
use tokio::time::sleep;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_unwrap::ResultExt;

use crate::{
    bot_trait::Bot,
    config::{get_config, get_shared_config},
    discord::DiscordBot,
    events::ServerMessage,
};

pub mod args;
pub mod bot_trait;
pub mod config;
pub mod events;
pub mod rabbitmq;
pub mod surreal;

#[cfg(feature = "discord")]
pub mod discord;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer().pretty())
        .with(EnvFilter::from_default_env())
        .init();
    let mut bots: Vec<Box<dyn Bot>> = vec![];

    let mut rabbitmq = Rabbitmq::new(&get_shared_config().await.rabbit_mq_url.to_string()).await;

    while let Err(_) = rabbitmq {
        error!("Failed to connect to rabbitmq, retrying in 5 secs...");
        sleep(Duration::from_secs(5)).await;
        rabbitmq = Rabbitmq::new(&get_shared_config().await.rabbit_mq_url.to_string()).await;
    }

    let rabbitmq = rabbitmq.unwrap();

    let QueueWithHandles(message_pair, _, _) = rabbitmq
        .connect_symmetrical::<ServerMessage>(get_config().await.services.clone())
        .await
        .unwrap_or_log();

    info!("Constructing bots...");
    #[cfg(feature = "discord")]
    {
        let message_pair = message_pair.clone();

        let mut dsbot = DiscordBot::default();
        dsbot.inject_pair(message_pair);

        bots.push(Box::new(dsbot));
    }

    surreal::init().await;

    for bot in bots {
        tokio::spawn(async move {
            info!("Starting service {}", bot.service_name());
            bot.start().await;
        });
    }

    loop {
        sleep(Duration::MAX).await;
    }
}
