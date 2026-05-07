use anyhow::Result;
use comrak::{markdown_to_html, ComrakOptions};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const ITALIC: &str = "\x1b[3m";
const STRIKE: &str = "\x1b[9m";
const BLUE: &str = "\x1b[34m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";

pub struct Parser {
    options: ComrakOptions,
}

impl Parser {
    pub fn new() -> Self {
        let mut options = ComrakOptions::default();
        options.extension.header_ids = Some("".to_string());
        options.extension.autolink = true;
        options.extension.table = true;
        options.extension.strikethrough = true;
        options.extension.tasklist = true;
        options.extension.superscript = true;
        options.extension.footnotes = true;
        options.extension.description_lists = true;
        options.render.unsafe_ = true;

        Self { options }
    }

    pub fn parse_to_html(&self, markdown: &str) -> Result<String> {
        let markdown = self.preprocess_callouts(markdown);
        let markdown = preprocess_math(&markdown);
        let html = markdown_to_html(&markdown, &self.options);
        Ok(html)
    }

    pub fn parse_to_plain(&self, markdown: &str) -> Result<String> {
        let plain = markdown
            .lines()
            .map(|line| {
                line.trim()
                    .trim_start_matches('#')
                    .trim_start_matches('>')
                    .trim_start_matches("- [ ]")
                    .trim_start_matches("- [x]")
                    .trim_start_matches("- ")
                    .trim_start_matches("* ")
                    .trim()
                    .to_string()
            })
            .filter(|line| !line.is_empty() && !line.starts_with("```"))
            .collect::<Vec<_>>()
            .join(" ");
        Ok(plain)
    }

    pub fn parse_to_terminal(&self, markdown: &str) -> Result<String> {
        let mut output = String::new();
        let mut in_code_block = false;

        for line in markdown.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                if in_code_block {
                    let lang = trimmed.trim_start_matches("```").trim();
                    if lang.is_empty() {
                        output.push_str(&format!("{DIM}┌─ code{RESET}\n"));
                    } else {
                        output.push_str(&format!("{DIM}┌─ {lang}{RESET}\n"));
                    }
                } else {
                    output.push_str(&format!("{DIM}└────{RESET}\n"));
                }
                continue;
            }

            if in_code_block {
                output.push_str(&format!("{CYAN}│ {line}{RESET}\n"));
                continue;
            }

            if trimmed.is_empty() {
                output.push('\n');
                continue;
            }

            if let Some((level, text)) = parse_heading(trimmed) {
                let marker = "●".repeat(level.min(3));
                output.push_str(&format!(
                    "{BOLD}{MAGENTA}{marker} {}{RESET}\n",
                    format_inline(text)
                ));
                continue;
            }

            if trimmed == "---" || trimmed == "***" || trimmed == "___" {
                output.push_str(&format!("{DIM}{}{RESET}\n", "─".repeat(72)));
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix("> ") {
                output.push_str(&format!("{DIM}│{RESET} {}\n", format_inline(rest)));
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix("- [x] ").or_else(|| trimmed.strip_prefix("- [X] ")) {
                output.push_str(&format!("{GREEN}☑{RESET} {}\n", format_inline(rest)));
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
                output.push_str(&format!("{YELLOW}☐{RESET} {}\n", format_inline(rest)));
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
                output.push_str(&format!("{BLUE}•{RESET} {}\n", format_inline(rest)));
                continue;
            }

            if is_ordered_item(trimmed) {
                output.push_str(&format!("{BLUE}{trimmed}{RESET}\n"));
                continue;
            }

            if trimmed.starts_with('|') && trimmed.ends_with('|') {
                output.push_str(&format!("{DIM}{}{RESET}\n", format_inline(trimmed)));
                continue;
            }

            if trimmed.starts_with(":::") {
                output.push_str(&format!("{YELLOW}{trimmed}{RESET}\n"));
                continue;
            }

            output.push_str(&format!("{}\n", format_inline(trimmed)));
        }

        Ok(output)
    }

    fn preprocess_callouts(&self, markdown: &str) -> String {
        let lines: Vec<&str> = markdown.lines().collect();
        let mut output = String::new();
        let mut index = 0;

        while index < lines.len() {
            let trimmed = lines[index].trim();
            if let Some(header) = trimmed.strip_prefix(":::") {
                let mut parts = header.trim().splitn(2, char::is_whitespace);
                let kind = parts.next().unwrap_or("").to_lowercase();
                let title = parts.next().map(str::trim).filter(|value| !value.is_empty());

                if matches!(kind.as_str(), "info" | "warn" | "warning" | "error" | "success" | "tip") {
                    index += 1;
                    let mut body = Vec::new();
                    while index < lines.len() && lines[index].trim() != ":::" {
                        body.push(lines[index]);
                        index += 1;
                    }
                    if index < lines.len() {
                        index += 1;
                    }

                    let rendered_body = markdown_to_html(&body.join("\n"), &self.options);
                    let title = title
                        .map(str::to_string)
                        .unwrap_or_else(|| kind.to_uppercase());
                    output.push_str(&format!(
                        "<div class=\"callout {kind}\"><div class=\"callout-title\">{}</div><div class=\"callout-body\">{rendered_body}</div></div>\n",
                        escape_html(&title)
                    ));
                    continue;
                }
            }

            output.push_str(lines[index]);
            output.push('\n');
            index += 1;
        }

        output
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_heading(line: &str) -> Option<(usize, &str)> {
    let level = line.chars().take_while(|ch| *ch == '#').count();
    if (1..=6).contains(&level) && line.chars().nth(level) == Some(' ') {
        Some((level, line[level + 1..].trim()))
    } else {
        None
    }
}

fn is_ordered_item(line: &str) -> bool {
    let Some((number, rest)) = line.split_once(". ") else {
        return false;
    };
    !number.is_empty() && number.chars().all(|ch| ch.is_ascii_digit()) && !rest.is_empty()
}

fn format_inline(input: &str) -> String {
    let linked = format_links(input);
    let code = apply_pair(&linked, "`", CYAN, RESET);
    let bold = apply_pair(&code, "**", BOLD, RESET);
    let italic = apply_pair(&bold, "*", ITALIC, RESET);
    apply_pair(&italic, "~~", STRIKE, RESET)
}

fn format_links(input: &str) -> String {
    let mut output = String::new();
    let mut rest = input;

    while let Some(start) = rest.find('[') {
        let before = &rest[..start];
        let after_start = &rest[start + 1..];
        let Some(end_text) = after_start.find("](") else {
            output.push_str(rest);
            return output;
        };
        let url_start = start + 1 + end_text + 2;
        let Some(end_url_rel) = rest[url_start..].find(')') else {
            output.push_str(rest);
            return output;
        };
        let text = &after_start[..end_text];
        let url = &rest[url_start..url_start + end_url_rel];

        output.push_str(before);
        output.push_str(text);
        output.push_str(&format!(" {DIM}({BLUE}{url}{RESET}{DIM}){RESET}"));
        rest = &rest[url_start + end_url_rel + 1..];
    }

    output.push_str(rest);
    output
}

fn apply_pair(input: &str, marker: &str, open: &str, close: &str) -> String {
    let mut output = String::new();
    let mut rest = input;
    let mut active = false;

    while let Some(index) = rest.find(marker) {
        output.push_str(&rest[..index]);
        output.push_str(if active { close } else { open });
        active = !active;
        rest = &rest[index + marker.len()..];
    }

    output.push_str(rest);
    if active {
        output.push_str(RESET);
    }
    output
}

fn preprocess_math(markdown: &str) -> String {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut output = String::new();
    let mut index = 0;

    while index < lines.len() {
        if lines[index].trim() == "$$" {
            index += 1;
            let mut body = Vec::new();
            while index < lines.len() && lines[index].trim() != "$$" {
                body.push(lines[index]);
                index += 1;
            }
            if index < lines.len() {
                index += 1;
            }
            output.push_str(&format!(
                "<div class=\"math-block\">{}</div>\n",
                escape_html(&body.join("\n"))
            ));
            continue;
        }

        output.push_str(&preprocess_inline_math(lines[index]));
        output.push('\n');
        index += 1;
    }

    output
}

fn preprocess_inline_math(line: &str) -> String {
    let mut output = String::new();
    let mut rest = line;

    while let Some(start) = rest.find('$') {
        output.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find('$') else {
            output.push_str(&rest[start..]);
            return output;
        };

        let math = after[..end].trim();
        if math.is_empty() {
            output.push_str("$$");
        } else {
            output.push_str(&format!("<span class=\"math-inline\">{}</span>", escape_html(math)));
        }
        rest = &after[end + 1..];
    }

    output.push_str(rest);
    output
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}