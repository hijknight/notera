/*!
# notera 📝

A powerful and lightweight CLI-based note-taking app built with [Rust](https://www.rust-lang.org/).

## 👣 Features

- 📋 Create, edit, delete, and view notes easily from your terminal using your favorite CLI editor (e.g., Vim (default), Nvim, Nano).
- 📂 Organize and sort notes effortlessly.
- 🕒 Timestamps for notes to track when they were created or updated.
- 🗑️ Clear all notes or delete them individually.
- 🗂️ Export notes to `.txt` or `.md` files for external use.
- 🤖 Includes a robust initialization and cleanup mechanism for managing configurations and data.
- 📦 Notes are safely stored using an SQLite database.
- 🚀 Fast and efficient workflow tailored for CLI enthusiasts.

## 📦 Installation

### Option 1: Clone Repository
1. Clone the repository:
   ```bash
   git clone https://github.com/hijknight/notera.git
   ```
2. Build **notera**:
   ```bash
   cd notera && cargo b -r
   ```
3. Add the `notera` command to your system PATH:

   **macOS/Linux**:
   ```bash
   sudo mv target/release/notera /usr/local/bin
   ```

   **Windows**:
   ```bash
   // Coming Soon 🔜
   ```

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
  - `export <FLAGS>`
    - Options:
      - `--all`: Export all notes into a single `.md` or `.txt` file
      - `--note`: Export a specific note into a `.md` or `.txt` file

  - `import <FLAGS>`
    - Options:
      - `--dir <DIR_PATH>`: Import all qualifying notes of a directory into notera
      - `--note <FILE_PATH>`: Import a specific note into notera

- Setup:
  - `config`: Open and modify the app's configuration settings.
    - Default config created after running `notera init`:
        ```toml
        editor = "nvim"
        note_db_directory = "/User/{user}/.local/share/notera"
        export_path = "/User/{user}/Documents/notera_exports"
        export_format = "md"

        # Possible values:

          # Editor: vim, nano, emacs, nvim, etc. (must be cli-based editor)

          # Note db directory: Should be kept default unless you know what you're doing.

          # Export path: Feel free to change, just make sure valid path.

          # IMPORTANT: Choose an export format. 'md' or 'txt'. md tends to be better for exports
        ```

  - `init`: Initialize `notera` for first-time use, setting up configurations and database storage.

  - `help`: Show the default help message.

- DANGER ZONE:
  - `clean`: Delete all temporary and persistent `notera` data (export files, , including the SQLite database and temporary files.


## 🛠 Configuration

The application automatically stores user preferences in a `config.toml` file for easier management. Open or modify it with the command:

```bash
notera config
```

Configuration options include the following:
- **Editor used**: The text editor used to create and edit notes (e.g., Vim).
- **Temporary Notes Directory**: Directory where temporary files are stored. (Should remain as default)
- **Export Path**: The directory location where exported files are saved.
- **Export Format**: The format in which exports are saved.

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


mod config;
mod storage;
mod setup;
mod error;
mod file_handling;

use std::process;
use clap::{ CommandFactory, Parser, Subcommand };

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
        #[arg(short, long, help = "Import all notes in a directory")]
        dir: Option<String>, // dir_path

        #[arg(long, value_name = "TITLE", help = "Import a specific note file. Provide format (.txt/.md) and file path")]
        note: Option<String>,
    },

    #[command(about = "export note(s)")]
    Export {
        #[arg(short, long, help = "Export all notes to a given format confugured in config.toml. Available formats: txt, md")]
        all: bool, // format

        #[arg(short, long, value_name = "TITLE", help = "Export a specific note. Provide format (.txt/.md) and note title")]
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
                let note_title = &args;
                storage::delete_note(note_title)
            } else {
                println!("No valid delete option provided. Use `--all` or `--note`.");
                Ok(())
            }
        },

        Some(Commands::Import {dir, note}) => {
            if let Some(dir) = dir {
                file_handling::import_dir(dir)
            } else if let Some(args) = note {
                if args.len() == 1 {
                    let file_path = &args;

                    match storage::init_db() {
                        Ok(conn) => {
                            match file_handling::import_note(&conn, file_path) {
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
                file_handling::export_note(title)
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
        error::handle_error(&error);
    }
}


