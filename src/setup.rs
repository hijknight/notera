use std::{
    env,
    process::Command,
    fs,
    io,
    path::Path
};
use crate::error::{
    NoteraError,
    Result,
    print_warning
};

pub fn is_initialized() -> bool {
    let config_path = crate::config::get_config_path();
    config_path.exists()
}

pub fn init() -> Result<()> {
    crate::config::Config::load()?;

    let config_path = crate::config::get_config_path();
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    let status = Command::new(&editor).arg(&config_path).status()
        .map_err(|e| NoteraError::Other(format!("Failed to open config file with editor: {}", e)))?;

    println!("✅ Config file created at {}", config_path.display());

    if !status.success() {
        return Err(NoteraError::Other("Config file opened with editor, but no changes were made.".to_string()));
    }
    let _db_conn = crate::storage::init_db()?;
    println!("✅ Database created at {}\n", crate::storage::get_db_path()?.display());
    println!("✅ notera initialized successfully!");

    Ok(())
}

pub fn open_config() -> Result<()> {
    let config_path = crate::config::get_config_path();
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    let status = Command::new(editor).arg(&config_path).status()
        .map_err(|e| NoteraError::Other(format!("Failed to open config file with editor: {}", e)))?;

    if !status.success() {
        return Err(NoteraError::Other("Config file opened with editor, but no changes were made.".to_string()));
    }
    Ok(())
}

/// Cleans up notera's data, including the SQLite database, and optionally the /notera folder.
pub fn clean() -> Result<()> {
    let db_path = crate::storage::get_db_path()?;
    let config_path = crate::config::get_config_path();
    let notera_files = crate::config::Config::load()?.notera_files_path;

    println!("⚠️ WARNING: This will delete the notera database, temporary files, config file, and optionally the notera files folder.\n\nType 'yes' to confirm:");
    let mut confirmation = String::new();

    io::stdin()
        .read_line(&mut confirmation)
        .map_err(|e| NoteraError::UserInput(format!("Failed to read user input: {}", e)))?;

    if confirmation.trim().to_lowercase() != "yes" {
        println!("❌ Clean operation aborted.");
        return Ok(());
    }

    // Delete the database file
    if db_path.exists() {
        match fs::remove_file(&db_path) {
            Ok(_) => println!("✅ Database file deleted: {}", db_path.display()),
            Err(e) => print_warning(&format!("Failed to delete database file: {}", e))
        }
    } else {
        println!("ℹ️ Database file not found. Nothing to clean there.");
    }


    if config_path.exists() {
        match fs::remove_file(&config_path) {
            Ok(_) => println!("✅ Configuration file deleted from {}.", config_path.display()),
            Err(e) => print_warning(&format!("Failed to delete configuration file: {}", e))
        }
    } else {
        println!("ℹ️ Configuration file not found. Nothing to delete.");
    }

    // optionally delete export folder.

    let mut export_deletion_confirmation = String::new();
    println!("🤷 Would you like to delete your notera export folder? (yes/no)");
    println!("ℹ️ Current notera files folder: {}", notera_files);

    io::stdin()
        .read_line(&mut export_deletion_confirmation)
        .map_err(|e| NoteraError::UserInput(format!("Failed to read user input: {}", e)))?;

    if export_deletion_confirmation.trim().to_lowercase() != "yes" {
        println!();
        println!("✅ Notera database deleted");
        println!("❎ Export folder not deleted per request.");
        println!();
        println!("✅ ❎ Clean operation completed.");
        return Ok(());
    }
    // if user says yes, delete all files
    let export_folder_exists: bool = Path::new(&notera_files).exists();

    if export_folder_exists {
        match fs::remove_dir_all(&notera_files) {
            Ok(_) => println!("✅ All exported files and the export directory deleted: {}", notera_files),
            Err(e) => print_warning(&format!("Failed to delete export directory: {}", e))
        }
    } else {
        println!("ℹ️ Export directory not found. Nothing to delete.")
    }

    println!();
    println!("✅ Notera database deleted");
    if !export_folder_exists {
        println!("ℹ️ Attempted to delete export folder, but it did not exist.");
    } else {
        println!("✅ Export folder deleted.");
    }

    println!();
    println!("✅ ✅ Clean operation completed.");
    Ok(())
}