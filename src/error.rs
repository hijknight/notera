
use std::{
    process::exit,
    fmt,
    io,
    path::PathBuf,
};
// exit codes

// 1 - standard exit
// 2 - db error exit
// 3 - fs error exit


#[derive(Debug)]
pub enum NoteraError {
    Database(rusqlite::Error),
    FileSystem(io::Error, Option<PathBuf>),
    Parse(String),
    Export(String),
    Import(String),
    UserInput(String),
    Other(String),
}

impl fmt::Display for NoteraError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NoteraError::Database(err) => write!(f, "Database error: {}", err),
            NoteraError::FileSystem(err, Some(path)) => write!(f, "File system error: {} at {}", err, path.display()),
            NoteraError::FileSystem(err, None) => write!(f, "File system error: {}", err),
            NoteraError::Parse(msg) => write!(f, "Parsing error: {}", msg),
            NoteraError::Export(msg) => write!(f, "Export error: {}", msg),
            NoteraError::Import(msg) => write!(f, "Import error: {}", msg),
            NoteraError::UserInput(msg) => write!(f, "User input error: {}", msg),
            NoteraError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl From<rusqlite::Error> for NoteraError {
    fn from(err: rusqlite::Error) -> Self {
        NoteraError::Database(err)
    }
}

impl From<io::Error> for NoteraError {
    fn from(err: io::Error) -> Self {
        NoteraError::FileSystem(err, None)
    }
}

impl From<toml::de::Error> for NoteraError {
    fn from(err: toml::de::Error) -> Self {
        NoteraError::Parse(err.to_string())
    }
}

impl From<toml::ser::Error> for NoteraError {
    fn from(err: toml::ser::Error) -> Self {
        NoteraError::Parse(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, NoteraError>;

pub fn handle_error(err: NoteraError) -> ! {
    match &err {
        NoteraError::Database(_) => handle_db_error(&err),
        NoteraError::FileSystem(_, _) => handle_fs_error(&err),
        NoteraError::Parse(_) => handle_parse_error(&err),
        NoteraError::Export(_) => handle_export_error(&err),
        NoteraError::Import(_) => handle_import_error(&err),
        NoteraError::UserInput(_) => handle_user_input_error(&err),
        NoteraError::Other(_) => handle_other_error(&err),
    }
}

const EXIT_GENERAL: i32 = 1;
const EXIT_DB_ERROR: i32 = 2;
const EXIT_FS_ERROR: i32 = 3;
const EXIT_PARSE_ERROR: i32 = 4;
const EXIT_EXPORT_ERROR: i32 = 5;
const EXIT_IMPORT_ERROR: i32 = 6;
const EXIT_USER_INPUT: i32 = 7;

pub fn handle_db_error(err: &NoteraError) -> ! {
    eprintln!("❌ {}", err);
    eprintln!("ℹ️ Please run `notera clean` to remove all files in /tmp/ and reset the database.");
    eprintln!("Then run `notera init` to reinitialize the database.");
    exit(EXIT_DB_ERROR);
}

pub fn handle_fs_error(err: &NoteraError) -> ! {
    eprintln!("❌ {}", err);
    eprintln!("ℹ️ Please check you file permissions.");
    exit(EXIT_FS_ERROR);
}

pub fn handle_parse_error(err: &NoteraError) -> ! {
    eprintln!("❌ {}", err);
    eprintln!("ℹ️ Please check the format of your input files.");
    exit(EXIT_PARSE_ERROR);
}

pub fn handle_export_error(err: &NoteraError) -> ! {
    eprintln!("❌ {}", err);
    eprintln!("ℹ️ Please check your export path and file permissions.");
    exit(EXIT_EXPORT_ERROR);
}

pub fn handle_import_error(err: &NoteraError) -> ! {
    eprintln!("❌ {}", err);
    eprintln!("ℹ️ Please check your import file format, permissions, and contents.");
    exit(EXIT_IMPORT_ERROR);
}

pub fn handle_user_input_error(err: &NoteraError) -> ! {
    eprintln!("❌ {}", err);
    exit(EXIT_USER_INPUT);
}

pub fn handle_other_error(err: &NoteraError) -> ! {
    eprintln!("❌ {}", err);
    exit(EXIT_GENERAL);
}

pub fn with_path(err: io::Error, path: PathBuf) -> NoteraError {
    NoteraError::FileSystem(err, Some(path))
}

pub fn print_warning(msg: &str) {
    eprintln!("⚠️ {}", msg);
}