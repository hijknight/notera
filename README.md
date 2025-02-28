# notera 📝

A powerful, lightweight, and simple CLI-based note-taking application built with [Rust](https://www.rust-lang.org/).

## 👣 Features

- 📋 Create, edit, delete, and view notes easily from your terminal using your favorite CLI editor (e.g., Vim, Nvim, Nano).
- 📂 Organize and sort notes effortlessly.
- 📦 Notes are safely stored using an SQLite database.
- 🚀 Fast and efficient workflow tailored for CLI enthusiasts.

## 📦 Installation

### Option 1: Clone repository
1. Clone
```bash
git clone https://github.com/hijknight/notera.git
```
2. Build notera
```bash
cd notera && cargo build --release
```

3. Add notera command to path

macOS:
```bash
sudo mv target/release/notera /usr/local/bin
```

Windows:
```rust
// Coming Soon 🔜
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

- Colorized tui 🟩 🟥 with [colored](https://crates.io/crates/colored) crate

## 🪪 License

This project is open-source and available under the MIT License
