use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use mindurka_rabbitmq_rust::{InjectQueues, QueuePair, RabbitReceiveMetadata, RabbitSendMetadata};
use poise::{
    Framework, FrameworkOptions, PrefixFrameworkOptions,
    samples::register_in_guild,
    serenity_prelude::{
        ClientBuilder, Context, EventHandler, ExecuteWebhook, GatewayIntents, GuildId, Http,
        Message, Webhook,
    },
};
use tokio::sync::broadcast;
use tracing::{error, info};
use tracing_unwrap::ResultExt;

use crate::{
    bot_trait::Bot, config::get_config, discord::commands::commands, events::ServerMessage,
};

pub mod commands;
pub mod config;
pub mod reports;

pub struct DiscordData {}
pub type DiscordError = Box<dyn std::error::Error + Send + Sync>; // TODO use thiserror crate
pub type DiscordContext<'a> = poise::Context<'a, DiscordData, DiscordError>;

#[derive(Default)]
pub struct DiscordBot {
    pub message_queue_pair:
        Option<QueuePair<RabbitReceiveMetadata<ServerMessage>, RabbitSendMetadata<ServerMessage>>>,
}

impl InjectQueues<RabbitReceiveMetadata<ServerMessage>, RabbitSendMetadata<ServerMessage>>
    for DiscordBot
{
    fn inject_pair(
        &mut self,
        queue_pair: QueuePair<
            RabbitReceiveMetadata<ServerMessage>,
            RabbitSendMetadata<ServerMessage>,
        >,
    ) {
        self.message_queue_pair = Some(queue_pair);
    }
}

#[async_trait]
impl Bot for DiscordBot {
    fn service_name(&self) -> &'static str {
        "Discord"
    }

    async fn start(&self) {
        let token = &get_config().await.discord.token;
        let intents = GatewayIntents::all();
        let guild_id = GuildId::new(get_config().await.discord.guild_id);

        let framework = Framework::builder()
            .options(FrameworkOptions {
                prefix_options: PrefixFrameworkOptions {
                    prefix: Some("$".into()),
                    ..Default::default()
                },
                commands: commands().await,
                ..Default::default()
            })
            .setup(move |ctx, _ready, framework| {
                Box::pin(async move {
                    let _ = register_in_guild(ctx, &framework.options().commands, guild_id).await;
                    Ok(DiscordData {})
                })
            })
            .build();

        let mut client_builder = ClientBuilder::new(token, intents).framework(framework);

        if let Some(pair) = &self.message_queue_pair {
            let handler = Handler { pair: pair.clone() };
            client_builder = client_builder.event_handler(handler);
        };

        let mut client = client_builder.await.unwrap_or_log();

        if let Some(QueuePair { broadcast, .. }) = &self.message_queue_pair {
            let http = client.http.clone();
            let mut broadcast = broadcast.resubscribe();
            tokio::spawn(async move {
                Self::message_handler(http, &mut broadcast).await;
            });
        };

        error!(
            "This should never happen, but ds bot stopped because: {:?}",
            client.start().await
        );
    }
}

struct Handler {
    pair: QueuePair<RabbitReceiveMetadata<ServerMessage>, RabbitSendMetadata<ServerMessage>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        if let Some((service, _)) = get_config()
            .await
            .discord
            .routes
            .iter()
            .find(|it| msg.channel_id == it.1.channel_id)
        {
            if msg.author.bot {
                return;
            }

            match self
                .pair
                .mpsc
                .send(RabbitSendMetadata {
                    service: service.clone(),
                    data: ServerMessage {
                        message: msg.content,
                        service: format!("{}@discord", msg.author.id),
                        user: Some("0".into()), /* TODO: really request id from db*/
                        username: msg
                            .author
                            .member
                            .as_ref()
                            .map(|it| it.nick.clone())
                            .unwrap_or(Some(msg.author.name.clone())),
                        avatar_url: msg.author.avatar_url(),
                    },
                })
                .await
            {
                Ok(_) => {}
                Err(err) => {
                    error!("Error while sending msg to rabbitmq: {}", err);
                }
            };
        };
    }
}

impl DiscordBot {
    async fn message_handler(
        http: Arc<Http>,
        queue: &mut broadcast::Receiver<RabbitReceiveMetadata<ServerMessage>>,
    ) {
        let mut webhooks = HashMap::<String, Webhook>::new();
        while let Ok(data) = queue.recv().await {
            let ServerMessage {
                message,
                service: _,
                user: _,
                username,
                avatar_url,
            } = &data.data;
            let sender = &data
                .metadata
                .app_id()
                .as_ref()
                .map(|it| it.as_str())
                .unwrap_or_default();
            let webhook = get_config()
                .await
                .discord
                .routes
                .get(*sender)
                .map(|it| &it.webhook_url);
            if webhook.is_none() {
                continue;
            };
            let webhook = webhook.unwrap();
            let webhook = match webhooks.get(webhook) {
                Some(webhook) => webhook,
                None => {
                    let webhook = Webhook::from_url(&Arc::clone(&http), &webhook)
                        .await
                        .unwrap();
                    &webhooks.entry(sender.to_string()).or_insert(webhook)
                }
            };
            let builder = ExecuteWebhook::new()
                .content(format!("`{}`", message.replace("`", "")))
                .username(username.as_ref().map_or("no name", |v| v))
                .avatar_url(avatar_url.as_ref().map_or("", |v| v));
            match webhook.execute(&http, false, builder).await {
                Ok(_) => {}
                Err(err) => {
                    error!("failed to send message using webhook: {}", err);
                }
            };
        }
    }
}

// pub mod commands;
// pub mod config;
// pub mod reports;

// use std::{collections::HashMap, ops::DerefMut, sync::Arc};

// use async_trait::async_trait;
// use poise::{
//     Framework, FrameworkOptions, PrefixFrameworkOptions,
//     samples::register_in_guild,
//     serenity_prelude::{
//         Client, ClientBuilder, Context, EventHandler, ExecuteWebhook, GatewayIntents, GuildChannel,
//         GuildId, Http, Message, Webhook,
//     },
// };
// use tokio::sync::{broadcast, mpsc};
// use tracing::{error, info};

// use crate::{
//     bot_trait,
//     config::get_config,
//     discord::{commands::commands, reports::main_report_handler},
//     events::{MindustryMessage, ReceivedMetadata, SendMetadata},
// };

// pub struct DiscordData {}
// pub type DiscordError = Box<dyn std::error::Error + Send + Sync>; // TODO use thiserror crate
// pub type DiscordContext<'a> = poise::Context<'a, DiscordData, DiscordError>;

// pub struct DiscordBot;

// impl DiscordBot {
//     pub fn new() -> DiscordBot {
//         Self
//     }

//     async fn from_mindustry_receiver(
//         http: Arc<Http>,
//         mut queue: broadcast::Receiver<ReceivedMetadata<MindustryMessage>>,
//     ) {
//         let mut webhooks = HashMap::<String, Webhook>::new();
//         while let Ok(msg) = queue.recv().await {
//             let sender = msg
//                 .rabbitmq_meta
//                 .app_id()
//                 .as_ref()
//                 .map(|it| it.as_str())
//                 .unwrap_or_default();
//             match msg.data {
//                 MindustryMessage::ServerMessage {
//                     message,
//                     service,
//                     user,
//                     username,
//                     avatar_url,
//                 } => {
//                     let webhook = get_config()
//                         .await
//                         .discord
//                         .routes
//                         .get(sender)
//                         .map(|it| it.webhook_url.clone())
//                         .unwrap_or_default();

//                     if webhook.is_empty() {
//                         continue;
//                     }

//                     let webhook = webhooks.entry(webhook.clone()).or_insert(
//                         Webhook::from_url(&http, &webhook)
//                             .await
//                             .expect("invalid webhook url"),
//                     );

//                     let builder = ExecuteWebhook::new()
//                         .content(format!("`{}`", message.replace("`", "")))
//                         .username(username.unwrap_or("no name".into()));
//                     match webhook.execute(&http, false, builder).await {
//                         Ok(_) => {}
//                         Err(err) => {
//                             error!("failed to send message using webhook: {}", err);
//                         }
//                     };
//                 }
//                 #[allow(unreachable_patterns)]
//                 _ => {}
//             }
//         }
//     }
// }

// #[async_trait]
// impl bot_trait::Bot for DiscordBot {
//     async fn run(
//         &mut self,
//         from_mindustry: broadcast::Receiver<ReceivedMetadata<MindustryMessage>>,
//         to_mindustry: mpsc::Sender<SendMetadata<MindustryMessage>>,
//     ) {
//         let token = &get_config().await.discord.token;
//         let intents = GatewayIntents::all();
//         let guild_id = GuildId::new(get_config().await.discord.guild_id);

//         let framework = Framework::builder()
//             .options(FrameworkOptions {
//                 prefix_options: PrefixFrameworkOptions {
//                     prefix: Some("$".into()),
//                     ..Default::default()
//                 },
//                 commands: commands().await,
//                 ..Default::default()
//             })
//             .setup(move |ctx, _ready, framework| {
//                 Box::pin(async move {
//                     let _ = register_in_guild(ctx, &framework.options().commands, guild_id).await;
//                     Ok(DiscordData {})
//                 })
//             })
//             .build();

//         let handler = Arc::new(MessageHandler {
//             queue: to_mindustry,
//         });
//         let mut client = ClientBuilder::new(token, intents)
//             .event_handler_arc(handler.clone())
//             .framework(framework)
//             .await
//             .unwrap();

//         let http1 = client.http.clone();

//         tokio::spawn(async move {
//             DiscordBot::from_mindustry_receiver(http1, from_mindustry).await;
//         });

//         error!(
//             "This should never happen, but ds bot stopped because: {:?}",
//             client.start().await
//         );
//     }
// }

// struct MessageHandler {
//     pub queue: mpsc::Sender<SendMetadata<MindustryMessage>>,
// }

// #[async_trait::async_trait]
// impl EventHandler for MessageHandler {
//     async fn message(&self, context: Context, msg: Message) {
//         if let Some((service, _)) = get_config()
//             .await
//             .discord
//             .routes
//             .iter()
//             .find(|it| msg.channel_id == it.1.channel_id)
//         {
//             if msg.author.bot {
//                 return;
//             }
//             let _ = self
//                 .queue
//                 .send(SendMetadata {
//                     data: MindustryMessage::ServerMessage {
//                         message: msg.content,
//                         service: format!("{}@discord", msg.author.id),
//                         user: Some("0".into()), /* TODO: really request id from db*/
//                         username: msg
//                             .author
//                             .member
//                             .as_ref()
//                             .map(|it| it.nick.clone())
//                             .unwrap_or(Some(msg.author.name.clone())),
//                         avatar_url: msg.author.avatar_url(),
//                     },
//                     target_service: service.clone(),
//                 })
//                 .await;
//         };
//     }

//     async fn thread_create(&self, ctx: Context, thread: GuildChannel) {
//         main_report_handler(ctx, thread).await;
//     }
// }
