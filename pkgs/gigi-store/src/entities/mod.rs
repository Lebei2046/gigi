//! Sea-ORM entities for gigi-store

pub mod message_acknowledgments;
pub mod messages;
pub mod offline_queue;

pub use message_acknowledgments::{
    Entity as MessageAcknowledgment, Model as MessageAcknowledgmentModel,
};
pub use messages::{Entity as Message, Model as MessageModel};
pub use offline_queue::{Entity as OfflineQueue, Model as OfflineQueueModel};
