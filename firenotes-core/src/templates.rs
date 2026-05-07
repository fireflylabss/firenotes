use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::Document;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: &'static str,
    pub emoji: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub content: String,
}

pub struct Templates;

impl Templates {
    pub fn all() -> Vec<Template> {
        let date = Utc::now().format("%Y-%m-%d").to_string();

        vec![
            Template {
                id: "meeting-notes",
                emoji: "🗓️",
                name: "Meeting Notes",
                description: "Agenda, attendees, action items, and decisions.",
                content: format!(
                    "# Meeting Notes\n\n**Date:** {date}\n**Project:** \n**Facilitator:** \n\n---\n\n## Attendees\n\n- \n\n## Agenda\n\n1. \n2. \n3. \n\n## Discussion\n\n### Topic 1\n\n_Notes here..._\n\n## Decisions Made\n\n- [ ] Decision 1\n- [ ] Decision 2\n\n## Action Items\n\n| Task | Owner | Due Date |\n| --- | --- | --- |\n|  |  |  |\n\n## Next Meeting\n\n**Date:** \n**Topics:**\n"
                ),
            },
            Template {
                id: "weekly-review",
                emoji: "📊",
                name: "Weekly Review",
                description: "Reflect on wins, blockers, learnings, and priorities.",
                content: format!(
                    "# Weekly Review\n\n**Week of:** {date}\n\n---\n\n## 🏆 Wins This Week\n\n- \n- \n\n## 🚧 Blockers & Challenges\n\n- \n- \n\n## 📚 Key Learnings\n\n- \n- \n\n## 📈 Metrics\n\n| Metric | Target | Actual |\n| --- | --- | --- |\n|  |  |  |\n\n## 🎯 Top 3 Priorities for Next Week\n\n1. \n2. \n3. \n\n## 💡 Ideas & Opportunities\n\n- \n\n## 🔋 Energy & Mood\n\n> Rate your week and add any personal notes.\n"
                ),
            },
            Template {
                id: "project-spec",
                emoji: "📐",
                name: "Project Spec",
                description: "Scope, goals, tech stack, architecture, and milestones.",
                content: format!(
                    "# Project Specification\n\n**Project Name:** \n**Version:** 1.0\n**Author:** \n**Date:** {date}\n\n---\n\n## Overview\n\n_A brief description of what this project does and why it exists._\n\n## Goals\n\n- \n- \n\n## Non-Goals\n\n- \n\n## Tech Stack\n\n| Layer | Technology |\n| --- | --- |\n| Frontend |  |\n| Backend |  |\n| Database |  |\n| Hosting |  |\n\n## Architecture\n\n_Describe the high-level architecture here._\n\n## Milestones\n\n| Milestone | Description | Due Date |\n| --- | --- | --- |\n| v0.1 Alpha |  |  |\n| v1.0 Release |  |  |\n\n## Open Questions\n\n- [ ] \n\n## References\n\n- \n"
                ),
            },
            Template {
                id: "daily-planner",
                emoji: "☀️",
                name: "Daily Planner",
                description: "Priorities, time blocks, quick captures, and reflection.",
                content: format!(
                    "# Daily Planner\n\n**{date}**\n\n---\n\n## 🌅 Morning Intentions\n\n> What would make today a great day?\n\n_Write here..._\n\n## 🎯 Top 3 Priorities\n\n- [ ] \n- [ ] \n- [ ] \n\n## 🕐 Time Blocks\n\n| Time | Task | Done |\n| --- | --- | --- |\n| 9:00 – 10:00 |  | ☐ |\n| 10:00 – 11:00 |  | ☐ |\n| 11:00 – 12:00 |  | ☐ |\n| 12:00 – 13:00 | 🍽️ Lunch |  |\n| 13:00 – 14:00 |  | ☐ |\n| 14:00 – 16:00 |  | ☐ |\n| 16:00 – 17:00 |  | ☐ |\n\n## 📝 Notes & Quick Captures\n\n- \n\n## 🌙 Evening Reflection\n\n**What went well?** \n\n**What could be improved?** \n\n**Gratitude:** \n"
                ),
            },
            Template {
                id: "bug-report",
                emoji: "🐛",
                name: "Bug Report",
                description: "Reproduction steps, environment details, and fix notes.",
                content: format!(
                    "# Bug Report\n\n**Bug ID:** BUG-\n**Severity:** 🔴 Critical / 🟠 High / 🟡 Medium / 🟢 Low\n**Reported by:** \n**Date:** {date}\n\n---\n\n## Summary\n\n_One-sentence description of the bug._\n\n## Steps to Reproduce\n\n1. Go to...\n2. Click on...\n3. Observe...\n\n## Expected Behavior\n\n_What should happen._\n\n## Actual Behavior\n\n_What actually happens._\n\n## Screenshots / Logs\n\n_Paste screenshots or error logs here._\n\n```text\nError output here\n```\n\n## Environment\n\n| Property | Value |\n| --- | --- |\n| OS |  |\n| Browser / Runtime |  |\n| App Version |  |\n| Node / Bun |  |\n\n## Possible Cause\n\n_Optional: any hypothesis about the root cause._\n\n## Fix Notes\n\n_Added after resolution._\n"
                ),
            },
            Template {
                id: "content-brief",
                emoji: "✍️",
                name: "Content Brief",
                description: "Plan posts, videos, outlines, audience, and SEO.",
                content: "# Content Brief\n\n**Title:** \n**Format:** Blog Post / Video / Thread / Newsletter\n**Author:** \n**Target Publish Date:** \n\n---\n\n## Goal\n\n_What should the reader/viewer take away from this piece?_\n\n## Target Audience\n\n- **Who are they?** \n- **Pain points:** \n- **What they know:** \n\n## Hook / Opening Line\n\n_Write a compelling first sentence._\n\n## Outline\n\n### Introduction\n- \n\n### Section 1: \n- \n\n### Section 2: \n- \n\n### Section 3: \n- \n\n### Conclusion / CTA\n- \n\n## SEO Keywords\n\n- Primary: \n- Secondary: \n\n## References & Research\n\n- \n\n## Notes\n\n_Any additional context for the writer._\n".to_string(),
            },
            Template {
                id: "retrospective",
                emoji: "🔄",
                name: "Sprint Retrospective",
                description: "What went well, improvements, ideas, and action items.",
                content: format!(
                    "# Sprint Retrospective\n\n**Sprint:** #\n**Date:** {date}\n**Team:** \n**Facilitator:** \n\n---\n\n## ✅ What Went Well\n\n- \n- \n\n## 🔧 What Could Be Improved\n\n- \n- \n\n## 💡 New Ideas & Experiments\n\n- \n- \n\n## 🚫 Stop Doing\n\n- \n\n## 📋 Action Items\n\n| Action | Owner | By When |\n| --- | --- | --- |\n|  |  |  |\n\n## 😊 Team Morale\n\n> Rate the team's overall energy this sprint (1–10) and add comments.\n\n**Score:** /10\n\n**Comments:** \n"
                ),
            },
            Template {
                id: "book-notes",
                emoji: "📖",
                name: "Book Notes",
                description: "Capture insights, quotes, action points, and references.",
                content: format!(
                    "# Book Notes\n\n**Title:** \n**Author:** \n**Rating:** ⭐⭐⭐⭐⭐\n**Finished:** {date}\n\n---\n\n## In One Sentence\n\n_The book in a single sentence._\n\n## Key Ideas\n\n### Idea 1: \n\n_Explanation..._\n\n### Idea 2: \n\n_Explanation..._\n\n### Idea 3: \n\n_Explanation..._\n\n## Favourite Quotes\n\n> \"Quote here.\"\n> — Author Name\n\n> \"Quote here.\"\n> — Author Name\n\n## What I'll Apply\n\n- [ ] \n- [ ] \n\n## Who Should Read This\n\n_Describe the ideal reader._\n\n## Further Reading\n\n- _Related book or resource_\n"
                ),
            },
        ]
    }

    pub fn find(id: &str) -> Option<Template> {
        Self::all().into_iter().find(|template| template.id == id)
    }

    pub fn create_document(id: &str) -> Option<Document> {
        Self::find(id).map(|template| Document::new(template.name.to_string(), template.content))
    }
}
