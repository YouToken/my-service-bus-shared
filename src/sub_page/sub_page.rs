use std::collections::BTreeMap;

use my_service_bus_abstractions::{queue_with_intervals::QueueWithIntervals, MessageId};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::messages::MySbMessageContent;

use super::{SizeAndAmount, SubPageId};

pub enum GetMessageResult<'s> {
    Message(&'s MySbMessageContent),
    Missing,
    GarbageCollected,
}

impl<'s> GetMessageResult<'s> {
    pub fn unwrap(&'s self) -> &'s MySbMessageContent {
        match self {
            GetMessageResult::Message(msg) => msg,
            GetMessageResult::Missing => panic!("Message is missing"),
            GetMessageResult::GarbageCollected => panic!("Message is garbage collected"),
        }
    }
}

pub struct SubPage {
    pub sub_page_id: SubPageId,
    pub messages: BTreeMap<i64, MySbMessageContent>,
    pub to_persist: QueueWithIntervals,
    pub created: DateTimeAsMicroseconds,
    pub garbage_collected: QueueWithIntervals,
    pub loaded: QueueWithIntervals,
    size_and_amount: SizeAndAmount,
}

impl SubPage {
    pub fn new(sub_page_id: SubPageId) -> Self {
        Self {
            sub_page_id,
            messages: BTreeMap::new(),
            created: DateTimeAsMicroseconds::now(),
            size_and_amount: SizeAndAmount::new(),
            to_persist: QueueWithIntervals::new(),
            garbage_collected: QueueWithIntervals::new(),
            loaded: QueueWithIntervals::new(),
        }
    }

    pub fn restore(sub_page_id: SubPageId, messages: BTreeMap<i64, MySbMessageContent>) -> Self {
        let mut size_and_amount = SizeAndAmount::new();
        let mut loaded = QueueWithIntervals::new();

        for msg in messages.values() {
            size_and_amount.added(msg.content.len());
            loaded.enqueue(msg.id.get_value());
        }

        Self {
            sub_page_id,
            messages,
            created: DateTimeAsMicroseconds::now(),
            size_and_amount,
            to_persist: QueueWithIntervals::new(),
            garbage_collected: QueueWithIntervals::new(),
            loaded,
        }
    }

    pub fn add_message(&mut self, message: MySbMessageContent) -> Option<MySbMessageContent> {
        self.size_and_amount.added(message.content.len());
        self.to_persist.enqueue(message.id.get_value());
        self.loaded.enqueue(message.id.get_value());

        if let Some(old_message) = self.messages.insert(message.id.get_value(), message) {
            self.size_and_amount.removed(old_message.content.len());
            return Some(old_message);
        }

        None
    }

    pub fn get_message(&self, msg_id: MessageId) -> GetMessageResult {
        if let Some(msg) = self.messages.get(msg_id.as_ref()) {
            return GetMessageResult::Message(msg);
        } else {
            if self.garbage_collected.has_message(msg_id.get_value()) {
                return GetMessageResult::GarbageCollected;
            }

            return GetMessageResult::Missing;
        }
    }

    pub fn get_size_and_amount(&self) -> &SizeAndAmount {
        &self.size_and_amount
    }

    pub fn get_messages_amount(&self) -> usize {
        self.messages.len()
    }

    fn get_first_message_id(&self) -> Option<MessageId> {
        for msg_id in self.messages.keys() {
            return Some(MessageId::new(*msg_id));
        }

        None
    }

    pub fn gc_messages(&mut self, min_message_id: MessageId) {
        let first_message_id = self.get_first_message_id();

        if first_message_id.is_none() {
            return;
        }

        let first_message_id = first_message_id.unwrap();

        for msg_id in first_message_id.get_value()
            ..self
                .sub_page_id
                .get_first_message_id_of_next_sub_page()
                .get_value()
        {
            if min_message_id.get_value() <= msg_id {
                break;
            }

            self.garbage_collected.enqueue(msg_id);
            if let Some(message) = self.messages.remove(&msg_id) {
                self.size_and_amount.removed(message.content.len());
            }
        }
    }

    pub fn persisted(&mut self, message_id: MessageId) {
        let _ = self.to_persist.remove(message_id.get_value());
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::messages::MySbMessageContent;
    #[test]
    fn test_gc_messages() {
        let mut sub_page = SubPage::new(SubPageId::new(0));

        sub_page.add_message(MySbMessageContent {
            id: 0.into(),
            content: vec![],
            time: DateTimeAsMicroseconds::now(),
            headers: None,
        });

        sub_page.add_message(MySbMessageContent {
            id: 1.into(),
            content: vec![],
            time: DateTimeAsMicroseconds::now(),
            headers: None,
        });

        sub_page.gc_messages(1.into());
    }
}
