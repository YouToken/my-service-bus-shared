use my_service_bus_abstractions::MessageId;

use super::MySbMessageContent;

#[derive(Debug, Clone)]
pub enum MySbMessage {
    Loaded(MySbMessageContent),
    Missing(MessageId),
}

impl MySbMessage {
    pub fn content_size(&self) -> usize {
        match self {
            MySbMessage::Loaded(msg) => msg.content.len(),

            MySbMessage::Missing(_) => 0,
        }
    }

    pub fn get_id(&self) -> MessageId {
        match self {
            MySbMessage::Loaded(msg) => msg.id,
            MySbMessage::Missing(id) => *id,
        }
    }

    pub fn is_garbage_collected(&self) -> bool {
        match self {
            MySbMessage::Loaded(_) => false,
            MySbMessage::Missing(_) => false,
        }
    }

    pub fn is_missing(&self) -> bool {
        match self {
            MySbMessage::Loaded(_) => false,
            MySbMessage::Missing(_) => true,
        }
    }

    pub fn unwrap_as_message(&self) -> &MySbMessageContent {
        match self {
            MySbMessage::Loaded(msg) => msg,
            MySbMessage::Missing(id) => panic!("Message {} is missing", id.get_value()),
        }
    }
}
