use anyhow::Result;

use crate::{Document, Parser};

pub struct Exporter;

impl Exporter {
    pub fn to_markdown(document: &Document) -> Result<String> {
        Ok(format!(
            "# {}\n\n{}\n\n---\n\n*Created: {}*\n\n*Updated: {}*\n\n*Tags: {}*",
            document.title,
            document.content,
            document.created_at.format("%Y-%m-%d %H:%M:%S"),
            document.updated_at.format("%Y-%m-%d %H:%M:%S"),
            document.tags.join(", ")
        ))
    }

    pub fn to_html(document: &Document) -> Result<String> {
        let markdown = Self::to_markdown(document)?;
        let html = comrak::markdown_to_html(&markdown, &comrak::ComrakOptions::default());
        Ok(html)
    }

    pub fn to_plain_text(document: &Document) -> Result<String> {
        Ok(format!(
            "{}\n\n{}\n\nCreated: {}\nUpdated: {}\nTags: {}",
            document.title,
            document.content,
            document.created_at.format("%Y-%m-%d %H:%M:%S"),
            document.updated_at.format("%Y-%m-%d %H:%M:%S"),
            document.tags.join(", ")
        ))
    }

    pub fn to_json(document: &Document) -> Result<String> {
        serde_json::to_string_pretty(document).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn to_terminal(document: &Document) -> Result<String> {
        let parser = Parser::new();
        parser.parse_to_terminal(&format!("# {}\n\n{}", document.title, document.content))
    }
}