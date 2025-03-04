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
    /// 📌 Note-taking commands
    #[command(subcommand_help_heading = "Note Commands")]
    #[command(about = "create a new note with `notera new <TITLE>`", alias = "n")]
    New { title: String },

    #[command(about = "list all notes with `notera list`", alias = "l")]
    List,

    #[command(about = "edit a specific note with `notera edit <TITLE>`", alias = "e")]
    Edit { title: String },

    #[command(about = "delete a specific note with `notera delete <TITLE>`", alias = "d")]
    Delete { title: String },

    #[command(about = "clear all notes with `notera clear`", alias = "c")]
    Clear,

    /// 🗂 Import and Export Commands
    #[command(subcommand_help_heading = "Import and Export Commands")]
    #[command(about = "import notes")]
    Import {
        #[arg(long, help = "Import all notes in a directory")]
        dir: Option<String>, // dir_path

        #[arg(long, num_args = 2, value_names = ["FORMAT", "TITLE"], help = "Import a specific note file. Provide format (.txt/.md) and file path")]
        note: Option<Vec<String>>,
    },

    #[command(about = "export note(s)")]
    Export {
        #[arg(long, value_name = "FORMAT", help = "Export all notes to a given format. .txt/.md")]
        all: Option<String>, // format

        #[arg(long, num_args = 2, value_names = ["FORMAT", "TITLE"], help = "Export a specific note. Provide format (.txt/.md) and note title")]
        note: Option<Vec<String>>,
    },

    /// Configuration and Setup
    #[command(about = "open the notera config file with `notera config`")]
    Config,

    #[command(about = "initialize notera with `notera init`")]
    Init,

    #[command(about = "DANGER: clean notera's data with `notera clean`")]
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

        Some(Commands::Clear) => storage::clear_notes(),


        Some(Commands::Import { dir, note }) => {
            if let Some(directory) = dir {
                storage::import_dir(&directory);
            } else if let Some(args) = note {
                if args.len() == 2 {
                    let format = &args[0];
                    let file_path = &args[1];
                    let _ = storage::import_note(&storage::init_db(), format, file_path);
                } else {
                    println!("❌ Invalid arguments for --note. Use: `notera import --note <FORMAT> <FILE_PATH>`");
                }
            } else {
                println!("❌ No valid import option provided. Use `--dir` or `--note`.");
            }
        }

        Some(Commands::Export { all, note }) => {
            if let Some(format) = all {
                storage::export_all(&format);
            } else if let Some(args) = note {
                if args.len() == 2 {
                    let format = &args[0];
                    let title = &args[1];
                    storage::export_note(&format, &title);
                } else {
                    println!("❌ Invalid arguments for --note. Use: `notera export --note <FORMAT> <TITLE>`");
                }
            } else {
                println!("❌ No valid export option provided. Use `--all` or `--note`.");
            }
        }


        Some(Commands::Config) => {
            setup::open_config()
        }

        Some(Commands::Init) => setup::init(),

        Some(Commands::Clean) => setup::clean(),

        None => {
            Cli::command().print_help().expect("Failed to display help")
        },
    }
}


