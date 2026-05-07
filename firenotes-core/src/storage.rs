use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use crate::Document;

pub struct Storage {
    base_path: PathBuf,
}

impl Storage {
    pub fn new(base_path: PathBuf) -> Result<Self> {
        fs::create_dir_all(&base_path)
            .context("Failed to create storage directory")?;
        Ok(Self { base_path })
    }

    pub fn default() -> Result<Self> {
        let firefly_path = PathBuf::from("/firefly/config/firenotes");
        let fallback_path = dirs::home_dir()
            .map(|p| p.join(".firenotes"))
            .unwrap_or_else(|| PathBuf::from(".firenotes"));

        let base_path = if firefly_path.exists() {
            firefly_path
        } else {
            fallback_path
        };

        Self::new(base_path)
    }

    fn document_path(&self, id: Uuid) -> PathBuf {
        self.base_path.join(format!("{}.json", id))
    }

    pub fn save(&self, document: &Document) -> Result<()> {
        let path = self.document_path(document.id);
        let json = serde_json::to_string_pretty(document)
            .context("Failed to serialize document")?;
        fs::write(&path, json)
            .context("Failed to write document")?;
        Ok(())
    }

    pub fn load(&self, id: Uuid) -> Result<Option<Document>> {
        let path = self.document_path(id);
        if !path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(&path)
            .context("Failed to read document")?;
        let document = serde_json::from_str(&json)
            .context("Failed to deserialize document")?;
        Ok(Some(document))
    }

    pub fn delete(&self, id: Uuid) -> Result<()> {
        let path = self.document_path(id);
        if path.exists() {
            fs::remove_file(&path)
                .context("Failed to delete document")?;
        }
        Ok(())
    }

    pub fn list_all(&self) -> Result<Vec<Document>> {
        let mut documents = Vec::new();

        for entry in fs::read_dir(&self.base_path)
            .context("Failed to read storage directory")?
        {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(json) = fs::read_to_string(&path) {
                    if let Ok(document) = serde_json::from_str::<Document>(&json) {
                        documents.push(document);
                    }
                }
            }
        }

        documents.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(documents)
    }

    pub fn search(&self, query: &str) -> Result<Vec<Document>> {
        let all = self.list_all()?;
        let query_lower = query.to_lowercase();

        let results: Vec<Document> = all
            .into_iter()
            .filter(|doc| {
                doc.title.to_lowercase().contains(&query_lower)
                    || doc.content.to_lowercase().contains(&query_lower)
                    || doc.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect();

        Ok(results)
    }

    pub fn replace_all(&self, query: &str, replacement: &str) -> Result<Vec<Document>> {
        let mut updated = Vec::new();

        for mut document in self.search(query)? {
            if document.replace_all(query, replacement) > 0 {
                self.save(&document)?;
                updated.push(document);
            }
        }

        Ok(updated)
    }
}