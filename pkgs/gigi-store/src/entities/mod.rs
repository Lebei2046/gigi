//! Sea-ORM entities for gigi-store

pub mod contacts;
pub mod conversations;
pub mod groups;
pub mod message_acknowledgments;
pub mod messages;
pub mod offline_queue;
pub mod settings;
pub mod shared_files;
pub mod thumbnails;

pub use contacts::Entity as Contacts;
pub use conversations::Entity as Conversation;
pub use groups::Entity as Groups;
pub use message_acknowledgments::Entity as MessageAcknowledgment;
pub use messages::Entity as Message;
pub use offline_queue::Entity as OfflineQueue;
pub use settings::Entity as Settings;
pub use shared_files::Entity as SharedFiles;
pub use thumbnails::Entity as Thumbnails;
