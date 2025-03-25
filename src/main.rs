/*!
# `notera` with AI features

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/hijknight/notera/rust.yml)
![Crates.io Version](https://img.shields.io/crates/v/notera)
![GitHub License](https://img.shields.io/github/license/hijknight/notera)


A powerful and lightweight CLI-based note-taking app built with [Rust](https://www.rust-lang.org/).

## 👣 Features

- 📋 Create, edit, delete, and view notes easily from your terminal using your favorite CLI editor (e.g., Vim (default), nvim, nano).
- 🤖 Summarize notes or lecture recordings with AI.
- 🕒 Timestamps for notes to track when they were created or updated.
- 🗂️ Export notes, summaries and transcripts to `.txt` or `.md` files for external use.
- 👌 Includes a robust initialization and cleanup mechanism for managing configurations and data.
- 📦 Notes are safely stored using an SQLite database.

## 📦 Installation

Please see [INSTALL.md](https://github.com/hijknight/notera/blob/master/INSTALL.md) for installation instructions and prerequisite information.


## 🏃‍♂️ Quick Start


Before running any commands, initialize `notera` for the first time:

```bash
notera init
```
This command will set up the required configurations (including storage paths) and initialize the SQLite database.


To check the available commands, run:

```bash
notera --help
```


## 💻 Supported CLI Actions
- 📝 Take notes:
  - `new <TITLE>`: Add a new note.

  - `view <FLAGS>`: View all or specific notes
    - Options:
      - `--all`: List all notes in database
      - `--note <TITLE>`: View the title, content and timestamp of a specific note

  - `edit <TITLE>`: Edit an existing note.

  - `delete <FLAGS>`: Delete all or a specific note
    - Options:
      - `--note <TITLE>`: Delete a specific note
      - `--all`: Delete all notes

- 🗂️ Exports and Imports:
  - `export <FLAGS> <ARGS>`
    - Options:
      - `--all`: Export all notes into a single `.md` or `.txt` file
      - `--note <TITLE>`: Export a specific note into a `.md` or `.txt` file

  - `import <FLAGS> <ARGS>`
    - Options:
      - `--dir <DIR_PATH>`: Import all qualifying notes of a directory into notera
      - `--note <FILE_PATH>`: Import a specific note into notera

- 🤖 AI use:
  - `summarize <FLAGS> <ARGS>`
    - Options:
      - `--note <TITLE>`: Summarize a specific `notera` note
      - `--file <FILE_PATH>`: Summarize a `.md` or `.txt` file
  - `transcribe <FLAGS>`
    - Options:
      - `--audio <AUDIO_FILE_PATH>`: transcribe an audio file
  - `lecture <FLAGS>`
    - Options:
      - `--audio <AUDIO_FILE_PATH>`: transcribe and summarize an audio file (targeted at lectures)

- Setup:
  - `config`: Open and modify the app's configuration settings.
    - Default config created after running `notera init`:
        ```toml
        editor = "nvim"
        note_db_directory = "/User/{user}/.local/share/notera"
        export_path = "/User/{user}/Documents/notera"
        export_format = "md"

        # Possible values:

          # Editor: vim, nano, emacs, nvim, etc. (must be cli-based editor)

          # Note db directory: Should be kept default unless you know what you're doing.

          # Export path: Feel free to change, just make sure of a valid path.

          # IMPORTANT: Choose an export format. 'md' or 'txt'. md tends to be better for exports
        ```

  - `init`: Initialize `notera` for first-time use, setting up configurations and database storage.

  - `help`: Show the default help message.

- DANGER ZONE:
  - `clean`: Delete all temporary and persistent `notera` data (export files, including the SQLite database and temporary files.)


## 🛠 Configuration

The application automatically stores user preferences in a `config.toml` file for easier management. Open or modify it with the command:

```bash
notera config
```

Configuration options include the following:
- **Editor used**: The text editor used to create and edit notes (e.g., Vim).
- **Database Directory**: Directory where the database is stored. (Should remain as default)
- **`notera` Files Path**: The directory location where exported files are saved.
- **Export Format**: The format in which exports, summaries, and transcripts are saved.

## 👷 Built With

- [Rust](https://www.rust-lang.org/) – for fast and safe application development.
- [serde](https://serde.rs/) & [toml-rs](https://github.com/alexcrichton/toml-rs) – data serialization and configuration parsing.
- [chrono](https://github.com/chronotope/chrono) – handling and formatting dates/timestamps.
- [rusqlite](https://github.com/rusqlite/rusqlite) – lightweight SQLite database library integration.
- [clap](https://github.com/clap-rs/clap) – parse and handle CLI arguments effortlessly.

## 🔮 Future Plans

- Better help message and optimizations for first release
- Listening with AI

## 🪪 License

This project is open-source and available under the MIT License.

*/

/// deals with config functions
mod config;
/// has all database functions (save, edit, delete)
mod storage;
/// contains clean and init functions
mod setup;
/// has error handling enum
mod error;
/// deals with imports and exports
mod file_handling;
/// handles all AI communications
mod ai;

use std::process;
use clap::{ CommandFactory, Parser, Subcommand };

/// `notera` CLI App
#[derive(Parser)]
#[command(name = "notera (AI-BETA)")]
#[command(version = "1.0.0-alpha")]
#[command(about = "A simple CLI-based note-taking app with powerful AI features.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, PartialEq)]
enum Commands {
    /// create a new file
    #[command(about = "create a new note with `notera new <TITLE>`", alias = "n")]
    New { title: String },

    /// view all notes or specify a title
    #[command(about = "view all notes with `notera view --all` or a specific notes `notera view --note <TITLE>`", alias = "l")]
    View {
        #[arg(short, long, help = "view all notes")]
        all: bool,

        #[arg(long, help = "view a note with a specific title")]
        note: Option<String>,
    },

    /// edit a specific note
    #[command(about = "edit a specific note with `notera edit <TITLE>`", alias = "e")]
    Edit { title: String },

    /// rename a note
    #[command(about = "rename a specific note with `notera rename <OLD_TITLE> <NEW_TITLE>`", alias = "r")]
    Rename { old_title: String, new_title: String },

    /// delete all or specific notes
    #[command(about = "delete a specific note with `notera delete <TITLE>`", alias = "d")]
    Delete {
        #[arg(short, long, help = "Delete a specific note with `notera delete <TITLE>`", value_name = "TITLE", alias = "d")]
        note: Option<String>,

        #[arg(short, long, help = "Delete all notes with `notera delete --all`")]
        all: bool,
    },

    /// import a directory of text files or a single file
    #[command(about = "import notes")]
    Import {
        #[arg(short, long, help = "Import all notes in a directory: <DIR_PATH>")]
        dir: Option<String>, // dir_path

        #[arg(short, long, value_name = "FILE_PATH", help = "Import a specific note file. Given a file path")]
        note: Option<String>,
    },

    /// export all current notes in database to a .txt or .md file or a specific note
    #[command(about = "export note(s)")]
    Export {
        #[arg(short, long, help = "Export all notes to a given format configured in config. Available formats: txt, md")]
        all: bool, // format

        #[arg(short, long, value_name = "TITLE", help = "Export a specific note. Provide format (.txt/.md) and note title")]
        note: Option<String>,
    },

    /// transcribe a given audio file and export it notera/transcripts
    #[command(about = "transcribe an audio file with openai's whisper")]
    Transcribe {
        #[arg(short, long, help = "tell notera whether to print the transcript to the terminal or not")]
        print: bool,
        #[arg(short, long, value_name = "AUDIO_FILE_PATH", help = "transcribe a given audio file (Only .m4a - Voice memos, quick time player)")]
        audio: Option<String>,
    },

    /// summarize a given notera note or a text file
    #[command(about = "summarize a given text file or note with ai")]
    Summarize {
        #[arg(short, long, help = "tell notera whether to export the summary to the notera/summaries folder or not")]
        export: bool,
        #[arg(short, long, help = "tell notera whether to print the summary to the terminal or not")]
        print: bool,
        #[arg(short, long, value_name = "FILE", help = "summarize a text file (.txt, .md)")]
        file: Option<String>,
        #[arg(short, long, value_name = "TITLE", help = "summarize a notera note given a title")]
        note: Option<String>,
        #[arg(short, long, value_name = "TEXT", help = "quickly summarize a piece of text")]
        text: Option<String>,
        #[arg(short, long, value_name = "FILE", help = "make a list nicer from a file (.txt, .md)")]
        list: Option<String>,
    },

    /// transcribe and summarize a recorded lecture
    #[command(about = "transcribe and summarize a lecture")]
    Lecture {
        #[arg(short, long, help = "tell notera whether to print the summary to the terminal or not")]
        print: bool,
        #[arg(short, long, value_name = "AUDIO_FILE_PATH", help = "transcribe and summarize a given audio file (.m4a - Voice memos, quick time player)")]
        audio: Option<String>,
    },

    /// change config
    #[command(about = "open the notera config file with `notera config`")]
    Config,

    /// initialize or reinitialize `notera` for first use
    #[command(about = "initialize notera with `notera init`")]
    Init,

    /// clean all notera files (optionally the Documents/notera folder)
    #[command(about = "DANGER: clean notera's data with `notera clean`")]
    Clean,
}


#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if !setup::is_initialized() && cli.command != Some(Commands::Init) {
        println!("❌ Notera not yet setup. Run `notera init` to initialize and set configuration options.");
        process::exit(1);
    }


    let result = match &cli.command {

        Some(Commands::New {title}) => storage::save_note(title),

        Some(Commands::View { all, note }) => {
            if *all {
                match storage::read_notes() {
                    Ok(notes) => {
                        println!("{}", notes.join("\n\n"));
                        Ok(())
                    },
                    Err(e) => Err(e),}
            } else if let Some(note) = note {
                match storage::read_note(note) {
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


        Some(Commands::Edit {title}) => storage::edit_note(title),

        Some(Commands::Rename {old_title, new_title}) => storage::rename_note(old_title, new_title),

        Some(Commands::Delete { note, all}) => {
            if *all {
                storage::clear_notes()
            } else if let Some(args) = note {
                let note_title = args;
                storage::delete_note(note_title)
            } else {
                println!("No valid delete option provided. Use `--all` or `--note`.");
                Ok(())
            }
        },

        Some(Commands::Import {dir, note}) => {
            if let Some(directory_given) = dir {
                file_handling::import_dir(directory_given)
            } else if let Some(note_file_path) = note {
                match storage::init_db() {
                    Ok(conn) => {
                        file_handling::import_note(&conn, note_file_path).unwrap_or_else(|err| {
                            println!("error importing file {}", err);
                            false
                        });
                        Ok(())
                    },
                    Err(e) => Err(e),
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
                let title = args;
                file_handling::export_note(title)
            } else {
                println!("❌ No valid export option provided. Use `--all` or `--note`.");
                Ok(())
            }
        },

        Some(Commands::Transcribe { audio , print}) => {
            if let Some(audio) = audio {
                let transcription_result = ai::Transcript::from_audio(audio).await;

                match transcription_result {
                    Ok(transcript) => {
                        if let Ok(_) = file_handling::export_transcript(&transcript) {
                            if *print {
                                transcript.print();
                            }

                            println!("✅ Transcript exported to {}/transcripts", config::Config::load().unwrap().notera_files_path);
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }

            Ok(())
        }

        Some(Commands::Summarize{ file, note, text , print, list, export}) => {
            if let Some(file) = file {
                if !*print && !*export {
                    println!("No flags given for summary to print or export. Use `--print` and/or `--export` to print or export the summary.");
                } else {
                    let summary_result = ai::Summary::from_file(file).await;

                    match summary_result {
                        Ok(summary) => {
                            if *print {
                                summary.print();
                            }

                            if *export {
                                if let Ok(_) = file_handling::export_summary(&summary) {
                                    println!("Summary exported to {}/summaries", config::Config::load().unwrap().notera_files_path);
                                } else {
                                    println!("Unable to export summary. Please check your permissions.");
                                }
                            }
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                }

                Ok(())
            } else if let Some(note) = note {
                if !*print && !*export {
                    println!("No flag given for summary to print or export. Use `--print` and/or `--export` to print or export the summary.");
                } else {
                    let summary_result = ai::Summary::from_note(note).await;

                    match summary_result {
                        Ok(summary) => {
                            if *print {
                                summary.print();
                            }

                            if *export {
                                if let Ok(_) = file_handling::export_summary(&summary) {
                                    println!("Summary exported to {}/summaries", config::Config::load().unwrap().notera_files_path);
                                } else {
                                    println!("Unable to export summary. Please check your permissions.");
                                }
                            }
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                }

                Ok(())
            } else if let Some(text) = text {
                if !*print && !*export {
                    println!("No flag given for summary to print or export. Use `--print` and/or `--export` to print or export the summary.");
                } else {
                    let summary_result = ai::Summary::from_text(text).await;

                    match summary_result {
                        Ok(summary) => {
                            if *print {
                                summary.print();
                            }

                            if *export {
                                if let Ok(_) = file_handling::export_summary(&summary) {
                                    println!("Summary exported to {}/summaries", config::Config::load().unwrap().notera_files_path);
                                } else {
                                    println!("Unable to export summary. Please check your permissions.");
                                }
                            }
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }

                }

                Ok(())
            } else if let Some(list) = list {
                if !*print && !*export {
                    println!("No flag given for summary to print or export. Use `--print` and/or `--export` to print or export the summary.");
                } else {
                    let summary_result = ai::Summary::from_list_file(list).await;
                    match summary_result {
                        Ok(summary) => {
                            if *print {
                                summary.print();
                            }

                            if *export {
                                if let Ok(_) = file_handling::export_summary(&summary) {
                                    println!("Summary exported to {}/summaries", config::Config::load().unwrap().notera_files_path);
                                } else {
                                    println!("Unable to export summary. Please check your permissions.");
                                }
                            }
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                }
                Ok(())
            } else {
                println!("No valid import option provided. Use `--file`, `--note`, `--text` or `--list`.");
                Ok(())
            }
        }

        Some(Commands::Lecture { audio , print }) => {
            if let Some(audio) = audio {
                let summary_result = ai::Summary::transcribe_and_summarize(audio).await;

                match summary_result {
                    Ok(summary) => {
                        if let Ok(_) = file_handling::export_summary(&summary) {

                            if *print {
                                summary.print();
                            }

                            println!("✅ Lecture summary exported to {}/summaries", config::Config::load().unwrap().notera_files_path);
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }

            Ok(())
        }

        Some(Commands::Config) => setup::open_config(),

        Some(Commands::Init) => setup::init(),

        Some(Commands::Clean) => setup::clean(),

        None => {
            Cli::command().print_help().map_err(|e| e.into())
        },
    };

    if let Err(error) = result {
        error::handle_error(&error);
    }
}


