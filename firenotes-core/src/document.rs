use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_HISTORY_ENTRIES: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySnapshot {
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub history: Vec<HistorySnapshot>,
}

impl Document {
    pub fn new(title: String, content: String) -> Self {
        Self::with_id(Uuid::new_v4(), title, content)
    }

    pub fn with_id(id: Uuid, title: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            content,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            history: Vec::new(),
        }
    }

    pub fn update_content(&mut self, content: String) {
        if self.content == content {
            return;
        }
        self.push_history_snapshot();
        self.content = content;
        self.updated_at = Utc::now();
    }

    pub fn update_title(&mut self, title: String) {
        self.title = title;
        self.updated_at = Utc::now();
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.updated_at = Utc::now();
        }
    }

    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.tags = tags;
        self.updated_at = Utc::now();
    }

    pub fn replace_all(&mut self, query: &str, replacement: &str) -> usize {
        if query.is_empty() {
            return 0;
        }

        let matches = self.content.matches(query).count();
        if matches > 0 {
            self.update_content(self.content.replace(query, replacement));
        }
        matches
    }

    pub fn restore_version(&mut self, history_index: usize) -> bool {
        let Some(snapshot) = self.history.get(history_index).cloned() else {
            return false;
        };
        self.update_content(snapshot.content);
        true
    }

    fn push_history_snapshot(&mut self) {
        if self
            .history
            .last()
            .map_or(false, |snapshot| snapshot.content == self.content)
        {
            return;
        }

        self.history.push(HistorySnapshot {
            content: self.content.clone(),
            timestamp: Utc::now(),
        });

        if self.history.len() > MAX_HISTORY_ENTRIES {
            let overflow = self.history.len() - MAX_HISTORY_ENTRIES;
            self.history.drain(0..overflow);
        }
    }

    pub fn word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }

    pub fn character_count(&self) -> usize {
        self.content.chars().count()
    }

    pub fn line_count(&self) -> usize {
        self.content.lines().count()
    }
}