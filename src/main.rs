mod storage;
mod config;
use clap::{ Parser, Subcommand, CommandFactory };
use std::process::Command;
use std::env;

/// Note-Taker CLI App
#[derive(Parser)]
#[command(name = "note-taker")]
#[command(version = "0.1.0.beta.0")]
#[command(about = "A simple CLI-based note-taking app", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    New { title: String },
    List,
    Edit { title: String },
    Delete { title: String },
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
        None => Cli::command().print_help().expect("Failed to display help"),
    }
}
