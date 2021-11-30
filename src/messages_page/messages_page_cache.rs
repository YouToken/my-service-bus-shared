use std::collections::{hash_map::Values, HashMap};

use crate::{messages::MySbMessageContent, page_id::PageId};

use super::MessagesPage;

pub struct PageInfo {
    pub page_no: PageId,
    pub page_size: usize,
    pub count: usize,
    pub persist_size: i64,
    pub is_being_persisted: bool,
}

pub struct MessagesPagesCache {
    pub pages: HashMap<PageId, MessagesPage>,
}

impl MessagesPagesCache {
    pub fn new() -> Self {
        Self {
            pages: HashMap::new(),
        }
    }

    pub fn new_messages(&mut self, page_id: PageId, msgs: Vec<MySbMessageContent>) {
        if !self.pages.contains_key(&page_id) {
            self.pages.insert(page_id, MessagesPage::new(page_id));
        }

        self.pages.get_mut(&page_id).unwrap().new_messages(msgs)
    }

    pub fn get_page(&self, page_id: PageId) -> Option<&MessagesPage> {
        self.pages.get(&page_id)
    }

    pub fn has_page(&self, page_id: &PageId) -> bool {
        self.pages.contains_key(&page_id)
    }

    pub fn get_pages_info(&self) -> Vec<PageInfo> {
        let mut result = Vec::new();

        for (page_id, page) in &self.pages {
            result.push(PageInfo {
                page_no: page_id.clone(),
                page_size: page.size,
                count: page.messages.len(),
                persist_size: page.to_be_persisted.len(),
                is_being_persisted: page.is_being_persisted,
            });
        }

        return result;
    }

    pub fn get_pages(&self) -> Values<PageId, MessagesPage> {
        self.pages.values()
    }

    pub fn remove_page(&mut self, page_id: &PageId) {
        self.pages.remove(page_id);
    }

    pub fn restore_page(&mut self, page: MessagesPage) {
        self.pages.insert(page.page_id, page);
    }

    pub fn get_persist_queue_size(&self) -> i64 {
        let mut result = 0;

        for page in self.pages.values() {
            result += page.to_be_persisted.len();
        }

        result
    }
}
