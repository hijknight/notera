mod config;
mod storage;
mod setup;
mod error;


use std::process;
use clap::{CommandFactory, Parser, Subcommand};

/// Note-Taker CLI App
#[derive(Parser)]
#[command(name = "notera")]
#[command(version = "0.1.0.beta.0")]
#[command(about = "A simple CLI-based note-taking app", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, PartialEq)]
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
    /// import a note in .txt or .md format with `notera import <FORMAT> <FILE_PATH>`
    Import { format: String, file_path: String },
    /// exports notes to a txt or md file ex: `notera export <FORMAT>`
    Export { format: String },
    /// opens config.toml in editor
    Config,
    /// intitializes or reinitializes notera
    Init,
    /// DANGER: deletes all notera data. Must run notera init to use again
    Clean,
}

fn main() {
    let cli = Cli::parse();

    if !setup::is_initialized() && cli.command != Some(Commands::Init) {
        println!("❌ Notera not yet setup. Run `notera init` to initialize and set configuration options.");
        process::exit(1);
    }


    match cli.command {
        Some(Commands::New { title }) => storage::save_note(&title),

        Some(Commands::List) => storage::read_notes().iter().for_each(|n| println!("{}", n)),

        Some(Commands::Edit { title }) => storage::edit_note(&title),

        Some(Commands::Delete { title }) => storage::delete_note(&title),

        Some(Commands::Config) => {
            setup::open_config()
        }

        Some(Commands::Clear) => storage::clear_notes(),

        Some(Commands::Import { format, file_path }) => storage::import_note(&format, &file_path),

        Some(Commands::Export { format }) => storage::export_notes(&format),

        Some(Commands::Init) => setup::init(),

        Some(Commands::Clean) => setup::clean(),

        None => {
            Cli::command().print_help().expect("Failed to display help")
        },
    }
}


