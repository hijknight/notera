# notera 📝
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/hijknight/notera/rust.yml)

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
   cd notera && cargo build --release
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
notera help
```


## 💻 Supported CLI Actions
- Take notes:
  - `notera new <TITLE>`: Add a new note.
  - `notera list`: List all notes with their titles, timestamps, and content snippets.
  - `notera view <TITLE>`: View the full content of a specific note (*automatically opens in CLI-defined text editor*).
  - `notera edit <TITLE>`: Edit an existing note.
  - `notera delete <TITLE>`: Delete a specific note.
  - `notera clear`: Permanently delete all notes.
  - `notera export <FORMAT>`: Export all notes to `.txt` or `.md` files.
      - Example: `notera export txt` will export notes into a txt file into specified directory configured with `notera config`
  - `notera import <FORMAT> <FILE_PATH>`: Import a note with a specified format (for in-house formatting) at a specified location.
  - `notera import-dir <FORMAT> <FILE_PATH>`: Coming soon

- Setup:
  - `notera config`: Open and modify the app's configuration settings.
  - `notera init`: Initialize `notera` for first-time use, setting up configurations and database storage.
  - `notera help`: Show the default help message.
- DANGER ZONE:
  - `notera clean`: Delete all temporary and persistent `notera` data (export files, , including the SQLite database and temporary files.


## 🛠 Configuration

The application automatically stores user preferences in a `config.toml` file for easier management. Open or modify it with the command:

```bash
notera config
```

Configuration options include the following:
- **Editor used**: The text editor used to create and edit notes (e.g., Vim).
- **Temporary Notes Directory**: Directory where temporary files are stored.
- **Export Path**: The directory location where exported files are saved.

## 👷 Built With

- [Rust](https://www.rust-lang.org/) – for fast and safe application development.
- [serde](https://serde.rs/) & [toml-rs](https://github.com/alexcrichton/toml-rs) – data serialization and configuration parsing.
- [chrono](https://github.com/chronotope/chrono) – handling and formatting dates/timestamps.
- [rusqlite](https://github.com/rusqlite/rusqlite) – lightweight SQLite database library integration.
- [clap](https://github.com/clap-rs/clap) – parse and handle CLI arguments effortlessly.

## 🔮 Future Plans

- 📂 Import notes as directories with `notera import-dir <FORMAT> <FILE_PATH>`
- Listening with AI

## 🪪 License

This project is open-source and available under the MIT License.
