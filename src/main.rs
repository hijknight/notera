mod config;
mod storage;
mod setup;
mod error;
mod file_handling;

use std::process;
use clap::{ CommandFactory, Parser, Subcommand };
use crate::error::handle_error;

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
    /// Note-taking commands
    #[command(about = "create a new note with `notera new <TITLE>`", alias = "n")]
    New { title: String },

    #[command(about = "view all notes with `notera view --all` or a specific notes `notera view --note <TITLE>`", alias = "l")]
    View {
        #[arg(short, long, help = "view all notes")]
        all: bool,

        #[arg(long, help = "view a note with a specific title")]
        note: Option<String>,
    },

    #[command(about = "edit a specific note with `notera edit <TITLE>`", alias = "e")]
    Edit { title: String },

    #[command(about = "rename a specific note with `notera rename <OLD_TITLE> <NEW_TITLE>`", alias = "r")]
    Rename { old_title: String, new_title: String },

    #[command(about = "delete a specific note with `notera delete <TITLE>`", alias = "d")]
    Delete {
        #[arg(short, long, help = "Delete a specific note with `notera delete <TITLE>`", value_name = "TITLE", alias = "d")]
        note: Option<String>,

        #[arg(short, long, help = "Delete all notes with `notera delete --all`")]
        all: bool,
    },

    /// 🗂 Import and Export Commands
    #[command(about = "import notes")]
    Import {
        #[arg(long, help = "Import all notes in a directory")]
        dir: Option<String>, // dir_path

        #[arg(long, value_name = "TITLE", help = "Import a specific note file. Provide format (.txt/.md) and file path")]
        note: Option<String>,
    },

    #[command(about = "export note(s)")]
    Export {
        #[arg(long, help = "Export all notes to a given format confugured in config.toml. Available formats: txt, md")]
        all: bool, // format

        #[arg(long, value_name = "TITLE", help = "Export a specific note. Provide format (.txt/.md) and note title")]
        note: Option<String>,
    },

    /// ⚙️ Configuration and Setup
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


    let result = match &cli.command {

        Some(Commands::New {title}) => storage::save_note(&title),

        Some(Commands::View { all, note }) => {
            if *all {
                match storage::read_notes() {
                    Ok(notes) => {
                        println!("{}", notes.join("\n\n"));
                        Ok(())
                    },
                    Err(e) => Err(e),}
            } else if let Some(note) = note {
                match storage::read_note(&note) {
                    Ok(notes) => {
                        println!("{}", notes.join("\n\n"));
                        Ok(())
                    },
                    Err(e) => Err(e),
                }
            } else {
                println!("No valid import option provided. Use `--all` or `--note`.");
                Ok(())
            }
        }


        Some(Commands::Edit {title}) => storage::edit_note(&title),

        Some(Commands::Rename {old_title, new_title}) => storage::rename_note(&old_title, &new_title),

        Some(Commands::Delete { note, all}) => {
            if *all {
                storage::clear_notes()
            } else if let Some(args) = note {
                let note_title = &args;
                storage::delete_note(&note_title)
            } else {
                println!("No valid delete option provided. Use `--all` or `--note`.");
                Ok(())
            }
        },

        Some(Commands::Import {dir, note}) => {
            if let Some(dir) = dir {
                file_handling::import_dir(&dir)
            } else if let Some(args) = note {
                if args.len() == 1 {
                    let file_path = &args;

                    match storage::init_db() {
                        Ok(conn) => {
                            match file_handling::import_note(&conn, &file_path) {
                                Ok(_) => Ok(()),
                                Err(e) => Err(e),
                            }
                        },
                        Err(e) => Err(e),
                    }


                } else {
                    println!("❌ No vald import option provided. Use: `notera import --note <FILE_PATH>`");
                    Ok(())
                }
            } else {
                println!("❌ No valid import option provided. Use `--dir` or `--note`.");
                Ok(())
            }
        }

        Some(Commands::Export { all, note }) => {
            if *all {
                file_handling::export_all()
            } else if let Some(args) = note {
                let title = &args;
                file_handling::export_note(&title)
            } else {
                println!("❌ No valid export option provided. Use `--all` or `--note`.");
                Ok(())
            }
        },

        Some(Commands::Config) => setup::open_config(),

        Some(Commands::Init) => setup::init(),

        Some(Commands::Clean) => setup::clean(),

        None => {
            Cli::command().print_help().map_err(|e| e.into())
        },
    };

    if let Err(error) = result {
        handle_error(&error);
    }
}


