use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::io::Read;

use firenotes_core::{Document, Exporter, Parser as MarkdownParser, Storage, Templates};

mod server;

#[derive(Parser)]
#[command(name = "firenotes")]
#[command(about = "FireNotes - Markdown editor for FireflyLabs", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new document
    New {
        /// Document title
        title: String,
        /// Template ID to use as initial content
        #[arg(short, long)]
        template: Option<String>,
    },
    /// List bundled templates
    Templates,
    /// Create a document from a bundled template
    Template {
        /// Template ID
        id: String,
    },
    /// List all documents
    List,
    /// Edit a document
    Edit {
        /// Document ID
        id: String,
    },
    /// Delete a document
    Delete {
        /// Document ID
        id: String,
    },
    /// Search documents
    Search {
        /// Search query
        query: String,
    },
    /// Export a document
    Export {
        /// Document ID
        id: String,
        /// Export format (markdown, html, plain, json, terminal)
        #[arg(short, long, default_value = "markdown")]
        format: String,
        /// Output file
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Show document details
    Show {
        /// Document ID
        id: String,
    },
    /// Render a document as Markdown in the terminal
    Preview {
        /// Document ID
        id: String,
    },
    /// Render Markdown from a file or stdin in the terminal
    Render {
        /// Markdown file path. Reads stdin when omitted.
        input: Option<String>,
    },
    /// List saved history snapshots for a document
    History {
        /// Document ID
        id: String,
    },
    /// Restore a document from a history snapshot
    Restore {
        /// Document ID
        id: String,
        /// History snapshot index
        index: usize,
    },
    /// Start the web UI
    Ui {
        /// Port to run on
        #[arg(short, long, default_value = "3000")]
        port: u16,
        /// Run in standalone mode (without core backend)
        #[arg(long)]
        standalone: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { title, template } => {
            let storage = Storage::default()?;
            let content = match template {
                Some(template_id) => Templates::find(&template_id)
                    .map(|template| template.content)
                    .with_context(|| format!("Template not found: {template_id}"))?,
                None => String::new(),
            };
            let document = Document::new(title.clone(), content);
            storage.save(&document)?;
            println!(
                "{} {} (ID: {})",
                "Created document:".green(),
                title,
                document.id
            );
        }
        Commands::Templates => {
            println!("{}", "Templates:".cyan());
            for template in Templates::all() {
                println!(
                    "  {} {} {} - {}",
                    "•".blue(),
                    template.emoji,
                    template.id,
                    template.description
                );
            }
        }
        Commands::Template { id } => {
            let storage = Storage::default()?;
            let document = Templates::create_document(&id)
                .with_context(|| format!("Template not found: {id}"))?;
            storage.save(&document)?;
            println!(
                "{} {} (ID: {})",
                "Created from template:".green(),
                document.title,
                document.id
            );
        }
        Commands::List => {
            let storage = Storage::default()?;
            let documents = storage.list_all()?;

            if documents.is_empty() {
                println!("{}", "No documents found.".yellow());
            } else {
                println!("{}", "Documents:".cyan());
                for doc in documents {
                    println!(
                        "  {} {} - {} ({} words)",
                        "•".blue(),
                        doc.title,
                        doc.id,
                        doc.word_count()
                    );
                }
            }
        }
        Commands::Edit { id } => {
            let storage = Storage::default()?;
            let uuid = uuid::Uuid::parse_str(&id)
                .context("Invalid document ID")?;
            
            match storage.load(uuid)? {
                Some(mut document) => {
                    println!("Editing: {}", document.title);
                    println!("Current content:\n{}", document.content);
                    println!("\nEnter new content (Ctrl+D to finish):");
                    
                    let mut new_content = String::new();
                    std::io::stdin().read_to_string(&mut new_content)?;
                    document.update_content(new_content);
                    
                    storage.save(&document)?;
                    println!("{}", "Document updated.".green());
                }
                None => println!("{}", "Document not found.".red()),
            }
        }
        Commands::Delete { id } => {
            let storage = Storage::default()?;
            let uuid = uuid::Uuid::parse_str(&id)
                .context("Invalid document ID")?;
            
            storage.delete(uuid)?;
            println!("{}", "Document deleted.".green());
        }
        Commands::Search { query } => {
            let storage = Storage::default()?;
            let results = storage.search(&query)?;

            if results.is_empty() {
                println!("{}", "No results found.".yellow());
            } else {
                println!("{}", "Search results:".cyan());
                for doc in results {
                    println!(
                        "  {} {} - {}",
                        "•".blue(),
                        doc.title,
                        doc.id
                    );
                }
            }
        }
        Commands::Export { id, format, output } => {
            let storage = Storage::default()?;
            let uuid = uuid::Uuid::parse_str(&id)
                .context("Invalid document ID")?;
            
            match storage.load(uuid)? {
                Some(document) => {
                    let content = match format.as_str() {
                        "markdown" => Exporter::to_markdown(&document)?,
                        "html" => Exporter::to_html(&document)?,
                        "plain" => Exporter::to_plain_text(&document)?,
                        "json" => Exporter::to_json(&document)?,
                        "terminal" => Exporter::to_terminal(&document)?,
                        _ => return Err(anyhow::anyhow!("Invalid format")),
                    };

                    match output {
                        Some(path) => {
                            std::fs::write(&path, content)?;
                            println!("{} {}", "Exported to:".green(), path);
                        }
                        None => {
                            println!("{}", content);
                        }
                    }
                }
                None => println!("{}", "Document not found.".red()),
            }
        }
        Commands::Preview { id } => {
            let storage = Storage::default()?;
            let uuid = uuid::Uuid::parse_str(&id)
                .context("Invalid document ID")?;

            match storage.load(uuid)? {
                Some(document) => {
                    print!("{}", Exporter::to_terminal(&document)?);
                }
                None => println!("{}", "Document not found.".red()),
            }
        }
        Commands::Render { input } => {
            let markdown = match input {
                Some(path) => std::fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read markdown file: {path}"))?,
                None => {
                    let mut buffer = String::new();
                    std::io::stdin().read_to_string(&mut buffer)?;
                    buffer
                }
            };
            let parser = MarkdownParser::new();
            print!("{}", parser.parse_to_terminal(&markdown)?);
        }
        Commands::History { id } => {
            let storage = Storage::default()?;
            let uuid = uuid::Uuid::parse_str(&id)
                .context("Invalid document ID")?;

            match storage.load(uuid)? {
                Some(document) => {
                    if document.history.is_empty() {
                        println!("{}", "No history snapshots found.".yellow());
                    } else {
                        println!("{}", "History:".cyan());
                        for (index, snapshot) in document.history.iter().enumerate() {
                            println!(
                                "  {} {} - {} chars",
                                index.to_string().blue(),
                                snapshot.timestamp,
                                snapshot.content.chars().count()
                            );
                        }
                    }
                }
                None => println!("{}", "Document not found.".red()),
            }
        }
        Commands::Restore { id, index } => {
            let storage = Storage::default()?;
            let uuid = uuid::Uuid::parse_str(&id)
                .context("Invalid document ID")?;

            match storage.load(uuid)? {
                Some(mut document) => {
                    if document.restore_version(index) {
                        storage.save(&document)?;
                        println!("{}", "Document restored.".green());
                    } else {
                        println!("{}", "History snapshot not found.".red());
                    }
                }
                None => println!("{}", "Document not found.".red()),
            }
        }
        Commands::Show { id } => {
            let storage = Storage::default()?;
            let uuid = uuid::Uuid::parse_str(&id)
                .context("Invalid document ID")?;
            
            match storage.load(uuid)? {
                Some(document) => {
                    println!("{}", "Document Details:".cyan());
                    println!("  Title: {}", document.title);
                    println!("  ID: {}", document.id);
                    println!("  Words: {}", document.word_count());
                    println!("  Characters: {}", document.character_count());
                    println!("  Lines: {}", document.line_count());
                    println!("  Tags: {}", document.tags.join(", "));
                    println!("  Created: {}", document.created_at);
                    println!("  Updated: {}", document.updated_at);
                    println!("\nContent:\n{}", document.content);
                }
                None => println!("{}", "Document not found.".red()),
            }
        }
        Commands::Ui { port, standalone } => {
            if standalone {
                println!("{} Running in standalone mode", 
                    "FireNotes UI:".green()
                );
                println!("Start the Svelte dev server with: cd firenotes-ui && bun run dev");
                println!("UI will use localStorage for storage");
            } else {
                println!("{} Starting core backend server on port {}", 
                    "FireNotes UI:".green(), 
                    port
                );
                println!("API available at: http://localhost:{}/api", port);
                println!("Start the Svelte dev server with: cd firenotes-ui && bun run dev");
                println!("Then access UI with: http://localhost:3000?mode=core&api=http://localhost:{}/api", port);
                
                server::run_server(port).await?;
            }
        }
    }

    Ok(())
}