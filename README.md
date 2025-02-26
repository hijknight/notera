# notera 📝

A powerful, lightweight, and simple CLI-based note-taking application built with [Rust](https://www.rust-lang.org/).

## 👣 Features

- 📋 Create, edit, delete, and view notes easily from your terminal using your favorite CLI editor (e.g., Vim, Nvim, Nano).
- 📂 Organize and sort notes effortlessly.
- 📦 Notes are safely stored using an SQLite database.
- 🚀 Fast and efficient workflow tailored for CLI enthusiasts.

## 📦 Installation

### 1. Clone the Repository
```bash
$ git clone <REPOSITORY_URL>
$ cd notera
```

### 2. Build and Run
```bash
$ cargo build --release
$ ./target/release/notera help
```

## 🏃‍♂️ Quick Start

To check the available commands, run:

```bash
$ notera help
```

## 💻 Supported CLI Actions

- `notera new <TITLE>`: Add a new note.
- `notera list`: List all notes.
- `notera edit <TITLE>`: Edit a specific note.
- `notera delete <TITLE>`: Delete a specific note.
- `notera config`: Set you favorite editor and notes directory.
- `notera help`: Show the default help message.

## 👷 Built With

- [Rust](https://www.rust-lang.org/) – for fast and safe application development.
- [serde](https://serde.rs/) & [toml-rs](https://github.com/alexcrichton/toml-rs) – data serialization and configuration parsing.
- [chrono](https://github.com/chronotope/chrono) – handling and formatting dates/timestamps.
- [rusqlite](https://github.com/rusqlite/rusqlite) – lightweight SQLite database library integration.

## 🔮 Future Plans

- 🔍 Advanced search functionality using fuzzy matching or `ripgrep`.

## 🪪 License

This project is open-source and available under the MIT License