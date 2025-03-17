# ⚠️This is a testing branch, a stable version of the beta [here](https://github.com/hijknight/notera)








![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/hijknight/notera/rust.yml)
![Crates.io Version](https://img.shields.io/crates/v/notera)
![GitHub License](https://img.shields.io/github/license/hijknight/notera)


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

Please see [INSTALL.md](https://github.com/hijknight/notera/blob/master/INSTALL.md) for installation instructions.


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
- **Database Directory**: Directory where the database is stored. (Should remain as default)
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
