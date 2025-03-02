mod config;
mod storage;
use clap::{CommandFactory, Parser, Subcommand};
use std::env;
use std::process::Command;

/// Note-Taker CLI App
#[derive(Parser)]
#[command(name = "notera")]
#[command(version = "0.1.0.alpha.0")]
#[command(about = "A simple CLI-based note-taking app", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// create a new note with `notera new <TITLE>`
    New { title: String },
    /// lists all notes
    List,
    /// edit and existing note with `notera edit <TITLE>`
    Edit { title: String },
    /// delete a specific note with `notera delete <TITLE>`
    Delete { title: String },
    /// clears all notes
    Clear,
    /// exports notes to a txt or md file
    Export {
        format: String,
        output_path: Option<String>,
    },
    /// opens config.toml in editor
    Config,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::New { title }) => storage::save_note(&title),

        Some(Commands::List) => storage::read_notes().iter().for_each(|n| println!("{}", n)),

        Some(Commands::Edit { title }) => storage::edit_note(&title),

        Some(Commands::Delete { title }) => storage::delete_note(&title),

        Some(Commands::Config) => {
            let config_path = config::get_config_path();
            let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
            let _ = Command::new(editor).arg(&config_path).status();
        }

        Some(Commands::Clear) => storage::clear_notes(),

        Some(Commands::Export {
            format,
            output_path,
        }) => storage::export_notes(&format, output_path.as_deref()),

        None => Cli::command().print_help().expect("Failed to display help"),
    }
}
