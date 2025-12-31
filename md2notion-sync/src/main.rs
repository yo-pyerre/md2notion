use anyhow::{Context, Result};
use clap::Parser;
use md2notion_core::parser::parse_markdown;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the markdown file to watch
    #[arg(short, long)]
    file: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = args.file;

    if !path.exists() {
        eprintln!("File does not exist: {:?}", path);
        std::process::exit(1);
    }

    println!("Watching {:?}", path);

    // Initial parse
    if let Err(e) = process_file(&path) {
        eprintln!("Error processing file: {:?}", e);
    }

    // Setup watcher
    let (tx, rx) = channel();
    
    let mut watcher = RecommendedWatcher::new(tx, Config::default())
        .context("Failed to create watcher")?;

    watcher.watch(&path, RecursiveMode::NonRecursive)
        .context("Failed to start watching file")?;

    for res in rx {
        match res {
            Ok(event) => {
                // We mainly care about content modification
                match event.kind {
                    notify::EventKind::Modify(_) | notify::EventKind::Create(_) => {
                        // Debounce could be added here, but for now direct processing
                        if let Err(e) = process_file(&path) {
                             eprintln!("Error processing file: {:?}", e);
                        }
                    }
                    _ => {}
                }
            }
            Err(e) => eprintln!("watch error: {:?}", e),
        }
    }

    Ok(())
}

fn process_file(path: &PathBuf) -> Result<()> {
    // Read file
    let content = std::fs::read_to_string(path).context("Failed to read file")?;
    
    // Parse
    let blocks = parse_markdown(&content).context("Failed to parse markdown")?;
    
    // Output JSON
    // We use pretty print for readability in this demo phase
    let json = serde_json::to_string_pretty(&blocks).context("Failed to serialize blocks")?;
    
    println!("{}", json);
    println!("----------------------------------------"); 
    
    Ok(())
}
