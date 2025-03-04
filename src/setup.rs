use std::{env, process::Command, fs, io, path::Path};

pub fn is_initialized() -> bool {
    let config_path = crate::config::get_config_path();
    config_path.exists()
}


pub fn init() {
    crate::config::Config::load();

    let config_path = crate::config::get_config_path();
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    let _ = Command::new(editor).arg(&config_path).status();
    println!("✅ Config file created at {}", config_path.display());

    crate::storage::init_db();
    println!("✅ Database created at {}\n", crate::storage::get_db_path().display());
    println!("✅ notera initialized successfully!");

}

pub fn open_config() {
    let config_path = crate::config::get_config_path();
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    let _ = Command::new(editor).arg(&config_path).status();
}



/// Cleans up notera's data, including the SQLite database, temp files, and optionally the config file.
pub fn clean() {
    let db_path = crate::storage::get_db_path();
    let config_path = crate::config::get_config_path();
    let export_path = crate::config::Config::load().export_path;
    let temp_dir = "/tmp/"; // Temporary directory path

    println!("⚠️ WARNING: This will delete the notera database, temporary files, config file, and optionally the notera_export folder.\n\nType 'yes' to confirm:");
    let mut confirmation = String::new();

    io::stdin()
        .read_line(&mut confirmation)
        .expect("Failed to read user input");

    if confirmation.trim().to_lowercase() != "yes" {
        println!("❌ Clean operation aborted.");
        return;
    }

    // Delete the database file
    if db_path.exists() {
        match fs::remove_file(&db_path) {
            Ok(_) => println!("✅ Database file deleted: {}", db_path.display()),
            Err(e) => eprintln!("❌ Failed to delete database file: {}", e),
        }
    } else {
        println!("ℹ️ Database file not found. Nothing to clean there.");
    }

    // Remove all temporary files
    if let Ok(entries) = fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(filename) = path.file_name() {
                if filename.to_string_lossy().starts_with("notera_") {
                    match fs::remove_file(&path) {
                        Ok(_) => println!("✅ Temporary file deleted: {}", path.display()),
                        Err(e) => eprintln!("❌ Failed to delete temporary file: {}", e),
                    }
                }
            }
        }
    } else {
        println!("ℹ️ Temporary directory not found or not accessible.");
    }

    if config_path.exists() {
        match fs::remove_file(&config_path) {
            Ok(_) => println!("✅ Configuration file deleted from {}.", config_path.display()),
            Err(e) => eprintln!("❌ Failed to delete configuration file: {}", e),
        }
    } else {
        println!("ℹ️ Configuration file not found. Nothing to delete.");
    }

    // delete exports

    // optionally delete export folder.

    let mut export_deletion_confirmation = String::new();
    println!("🤷 Would you like to delete your notera export folder? (yes/no)");

    io::stdin()
        .read_line(&mut export_deletion_confirmation)
        .expect("Failed to read user input");

    if export_deletion_confirmation.trim().to_lowercase() != "yes" {
        println!();
        println!("✅ All files in '{}' prefixed with 'notera_' deleted.", temp_dir);
        println!("✅ Notera database deleted");
        println!("❌ Export folder not deleted per request.");
        println!();
        println!("✅ ✅ ❎ Clean operation completed.");
        return;
    }
    // if user says yes, delete all files
    let export_folder_exists: bool = Path::new(&export_path).exists();

    if export_folder_exists {
        match fs::remove_dir_all(&export_path) {
            Ok(_) => println!("✅ All exported files and the export directory deleted: {}", export_path),
            Err(e) => eprintln!("❌ Failed to delete export files or folder: {}", e),
        }
    } else {
        println!("ℹ️ Export directory not found. Nothing to delete.")
    }

    println!();
    println!("✅ All files in '{}' prefixed with 'notera_' deleted.", temp_dir);
    println!("✅ Notera database deleted");
    if !export_folder_exists {
        println!("ℹ️ Attempted to delete export folder, but it did not exist.");
    } else {
        println!("✅ Export folder deleted.");
    }

    println!();
    println!("✅ ✅ ✅ Clean operation completed.");
}