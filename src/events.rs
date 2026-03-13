use mindurka_rabbitmq_rust::NetworkEvent;
use mindurka_rabbitmq_rust::network_event;
use serde::{Deserialize, Serialize};

#[network_event(queue = "generic.message")]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerMessage {
    /// Plaintext message
    pub message: String,
    /// Address of the server
    pub service: String,
    /// Shared user id
    pub user: Option<String>,
    /// Displayed username
    pub username: Option<String>,
    /// Displayed avatar
    pub avatar_url: Option<String>,
}

// use lapin::protocol::basic::AMQPProperties;
// use serde::{Deserialize, Serialize};

// #[derive(Debug, Clone)]
// pub struct ReceivedMetadata<T> {
//     pub data: T,
//     pub rabbitmq_meta: AMQPProperties,
// }

// #[derive(Debug, Clone)]
// pub struct SendMetadata<T> {
//     pub data: T,
//     pub target_service: String,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(untagged)]
// #[serde(rename_all_fields = "camelCase")]
// pub enum MindustryMessage {
//     ServerMessage {
//         /// Plaintext message
//         message: String,
//         /// Address of the server
//         service: String,
//         /// Shared user id
//         user: Option<String>,
//         /// Displayed username
//         username: Option<String>,
//         /// Displayed avatar
//         avatar_url: Option<String>,
//     },
// }

// impl MindustryMessage {
//     #[inline(always)]
//     pub const fn queue_name(&self) -> &str {
//         match self {
//             Self::ServerMessage { .. } => "generic.message",
//         }
//     }
// }
