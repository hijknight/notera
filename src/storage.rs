use rusqlite::{ params, Connection };
use std::fs;
use std::io::Write;
use std::path::{ PathBuf, Path };
use crate::config::Config;
use chrono::Local;
use crate::error::{print_warning, with_path, NoteraError, Result};

pub fn get_db_path() -> Result<PathBuf> {
    let config = Config::load()?;
    let mut path = PathBuf::from(config.note_db_directory);
    fs::create_dir_all(&path).map_err(|e| with_path(e, path.clone()))?;
    path.push("notes.db");
    Ok(path)
}

pub fn init_db() -> Result<Connection> {
    let db_path = get_db_path()?;
    let conn = Connection::open(&db_path)
        .map_err(|e| NoteraError::Database(e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            timestamp TEXT NOT NULL
        )",
        [],
    ).map_err(|e| NoteraError::Database(e))?;

    Ok(conn)
}


pub fn save_note(title: &str) -> Result<()> {
    let conn = init_db()?;
    let config = Config::load()?;
    let editor = &config.editor;

    let temp_file_path = format!("/tmp/notera_{}.md", title.replace(" ", "_"));

    // open temp file
    let status = std::process::Command::new(&editor)
        .arg(&temp_file_path)
        .status()
        .map_err(|e| NoteraError::Other(format!("Failed to open editor: {}", e)))?;


    if !status.success() {
        return Err(NoteraError::Other("Editor exited with non-zero status".to_string()));
    }
    // read note content
    let content = fs::read_to_string(&temp_file_path).map_err(|e| with_path(e, PathBuf::from(&temp_file_path)))?
        .trim()
        .to_string();

    if content.is_empty() {
        println!("⚠️ Note discarded (empty content).");
        return Ok(());
    }

    let timestamp = Local::now().format("%Y-%m-%d %H:%M").to_string();

    conn.execute(
        "INSERT INTO notes (title, content, timestamp) VALUES (?1, ?2, ?3)",
        params![title.trim(), content, timestamp],
    ).map_err(|e| NoteraError::Database(e))?;



    println!("✅ Note saved successfully!");
    Ok(())
}


/// Read all existing notes
pub fn read_notes() -> Result<Vec<String>> {

    let conn = init_db()?;

    let mut stmt = conn.prepare("SELECT title, content, timestamp FROM notes ORDER BY timestamp DESC")
        .map_err(|e| NoteraError::Database(e))?;

    let notes_iter = stmt.query_map([], |row| {
        let title: String = row.get(0)?;
        let content: String = row.get(1)?;
        let timestamp: String = row.get(2).unwrap_or_else(|_| "Unknown Timestamp".to_string()); // Handle missing column

        Ok((title, content, timestamp))
    }).map_err(|e| NoteraError::Database(e))?;


    let mut notes = Vec::new();

    for note_result in notes_iter {
        let (title, content, timestamp) = note_result.map_err(|e| NoteraError::Database(e))?;
        notes.push(format!("📝 Title: {}\n⏳ Created: {}\n\n{}\n", title, timestamp, content));
    }
    Ok(notes)
}
/// Edit an existing note
pub fn edit_note(title: &str) -> Result<()> {
    let conn = init_db()?;
    let config = Config::load()?;
    let editor = &config.editor;

    let mut stmt = conn.prepare("SELECT content FROM notes WHERE title = ?1")
        .map_err(|e| NoteraError::Database(e))?;

    let content_result = stmt.query_row(params![title], |row| row.get(0));

    let content: String = match content_result {
        Ok(content) => content,
        Err(_) => {
            print_warning(&format!("Note not found: '{}'", title));
            return Ok(());
        }
    };

    if content.is_empty() {
        print_warning(&format!("Note not found: '{}'", title));
        return Ok(());
    }


    let temp_file_path = format!("/tmp/notera_{}.md", title.replace(" ", "_"));

    fs::write(&temp_file_path, &content)
        .map_err(|e| with_path(e, PathBuf::from(&temp_file_path)))?;

    let status = std::process::Command::new(&editor)
        .arg(&temp_file_path)
        .status()
        .map_err(|e| NoteraError::Other(format!("Failed to open editor: {}", e)))?;

    if !status.success() {
        return Err(NoteraError::Other("Editor exited with non-zero status".to_string()));
    }

    let updated_content = fs::read_to_string(&temp_file_path)
        .map_err(|e| with_path(e, PathBuf::from(&temp_file_path)))?
        .trim()
        .to_string();

    if updated_content.is_empty() {
        println!("⚠️ No changes made.");
        return Ok(());
    }

    let timestamp = Local::now().format("%Y-%m-%d %H:%M").to_string();

    conn.execute(
        "UPDATE notes SET content = ?1, timestamp = ?2 WHERE title = ?3",
        params![updated_content, timestamp, title],
    ).map_err(|e| NoteraError::Database(e))?;

    println!("✅ Note updated successfully!");
    Ok(())
}

pub fn delete_note(title: &str) -> Result<()> {
    let conn = init_db()?;
    let title = title.trim();
    let result = conn.execute("DELETE FROM notes WHERE title = ?1", params![title])
        .map_err(|e| NoteraError::Database(e))?;
    // I forgot an s in notes, and it took me an hour to fix. ^^^ OMG

    if result == 0 {
        print_warning(&format!("Note not found: '{}'", title));
        return Ok(())
    }

    println!("✅ Note '{}' deleted from database.", title);

    let temp_file_path = format!("/tmp/notera_{}.md", title.replace(" ", "_"));

    if let Err(e) = fs::remove_file(&temp_file_path) {
        print_warning(&format!("Failed to delete temporary file: {}", e));
        println!("ℹ️ Normally, this happens because the file was imported, so no file was created in the /tmp directory.");
    } else {
        println!("✅ Temporary file deleted.");
    }

    Ok(())
}

// Clears all notes from db and tmp directory
pub fn clear_notes() -> Result<()> {
    let conn = init_db()?;


    println!("⚠️ WARNING: This will permanently delete all notes. Type 'yes' to confirm");
    let mut confirmation = String::new();

    std::io::stdin()
        .read_line(&mut confirmation)
        .map_err(|e| NoteraError::UserInput(format!("Failed to read confirmation: {}", e)))?;

    let confirmation = confirmation.trim().to_lowercase();

    if confirmation != "yes" {
        println!("❌ Clear operation aborted");
        return Ok(());
    }

    conn.execute("DELETE FROM notes", params![]).map_err(|e| NoteraError::Database(e))?;

    println!("✅ All notes deleted from database.");

    // delete temp files
    let temp_file_path = "/tmp/";

    if let Ok(entries) = fs::read_dir(temp_file_path) {
        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.starts_with("notera_") {
                    let _ = fs::remove_file(entry.path()).unwrap_or_else(|err| {
                        println!("❌ Failed to delete tmp file: {}", err);
                        println!("ℹ️ Normally, this happens because the file was imported, so no file was created in the /tmp directory.")
                    });
                }
            }
        }
    }

    println!("✅ Temporary note files cleared.");
    Ok(())
}


pub fn export_all(format: &str) -> Result<()>{
    let conn = init_db()?;
    let mut stmt = conn.prepare("SELECT title, content, timestamp FROM notes ORDER BY timestamp DESC")
        .map_err(|e| NoteraError::Database(e))?;
    let config = Config::load()?;

    let export_path = &config.export_path;



    //early check
    if format != "txt" && format != "md" {
        print_warning("Unsupported format. Please use txt or md.");
        return Ok(());
    }


    if !fs::exists(&export_path)? {
        fs::create_dir_all(&export_path)
            .map_err(|e| NoteraError::FileSystem(e, None))?;
    }

    let notes_iter = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    }).map_err(|e| NoteraError::Database(e))?;

    let mut notes = Vec::new();

    for note in notes_iter {
        let (title, content, timestamp) = note?;

        notes.push((title, content, timestamp));
    }

    if notes.is_empty() {
        print_warning("No notes found. Nothing to export.");
        return Ok(());
    }

    let export_timestamp = Local::now().format("%Y-%m-%d_%H.%M").to_string();
    let default_filename = format!("notera-export_all-{}.{}", export_timestamp, format);
    let output_path = format!("{}/{}", export_path, default_filename);

    let mut file = fs::File::create(&output_path)
        .map_err(|e| NoteraError::Export(format!("Failed to create file: {}", e)))?;


    match format {
        "txt" => {
            for (title, content, timestamp) in &notes {
                writeln!(file, "Title: {}\n\nCreated: {}\n-----\n\n{}\n-----------", title, timestamp, content)
                    .map_err(|e| NoteraError::Export(format!("Failed to write to file: {}", e)))?;
            }
        },
        "md" => {
            writeln!(file, "# notera markdown export {}\n", export_timestamp)
                .map_err(|e| NoteraError::Export(format!("Failed to write to file: {}", e)))?;
            for (title, content, timestamp) in &notes {
                writeln!(file, "## 📝 {}\n\n#### ⏳ *Created: {}*\n\n{}\n---\n", title, timestamp, content)
                    .map_err(|e| NoteraError::Export(format!("Failed to write to file: {}", e)))?;
            }
        },
        // catch all pattern to satisfy compiler. will never be run.
        _ => {
            print_warning("You are officially a wizard. This could should have been unreachable.");
            return Ok(());
        },
    }

    println!("✅  Notes exported successfully to {}", output_path);
    Ok(())
}

pub fn export_note(format: &str, title_query: &str) -> Result<()> {
    let conn = init_db()?;

    let mut stmt = conn.prepare("SELECT title, content, timestamp FROM notes WHERE title = ?1").expect("❌ Note not found in database.");

    let config = Config::load()?;

    let export_path = &config.export_path;

    if format != "txt" && format != "md" {
        print_warning("Unsupported format. Please use txt or md.");
        return Ok(());
    }

    if !fs::exists(&export_path)? {
        fs::create_dir_all(&export_path)
            .map_err(|e| NoteraError::FileSystem(e, None))?;
    }

    let notes_iter = stmt.query_map(params![title_query], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    }).map_err(|e| NoteraError::Database(e))?;

    let mut notes = Vec::new();

    for note in notes_iter {
        let (title, content, timestamp) = note?;

        notes.push((title, content, timestamp));
    }

    let export_timestamp = Local::now().format("%Y-%m-%d_%H.%M").to_string();
    let default_filename = format!("notera-export_{}-{}.{}", title_query, export_timestamp, format);
    let output_path = format!("{}/{}", export_path, default_filename);

    let mut file = fs::File::create(&output_path)
        .map_err(|e| NoteraError::Export(format!("Failed to create file: {}", e)))?;

    match format {
        "txt" => {
            for (title, content, timestamp) in &notes {
                writeln!(file, "Title: {}\n\nCreated: {}\n-----\n\n{}\n-----------", title, timestamp, content)
                    .map_err(|e| NoteraError::Export(format!("Failed to write to file: {}", e)))?;
            }
        }
        "md" => {
            writeln!(file, "# notera markdown export {} for note {}\n", export_timestamp, title_query)
                .map_err(|e| NoteraError::Export(format!("Failed to write to file: {}", e)))?;
            for (title, content, timestamp) in &notes {
                writeln!(file, "## 📝 {}\n\n#### ⏳ *Created: {}*\n\n{}\n\n", title, timestamp, content)
                    .map_err(|e| NoteraError::Export(format!("Failed to write to file: {}", e)))?;
            }
        }
        // the pattern below will never be able to be run, because of the check above.
        _ => {
            print_warning("You are officially a wizard. This could should have been unreachable.");
            return Ok(());
        }
    }
    println!();
    println!("✅ Note exported successfully to {} of {}", output_path, title_query);
    Ok(())
}

pub fn import_note(conn: &Connection, format: &str, file_path: &str) -> Result<bool> {

    if format != "txt" && format != "md" {
        print_warning("Unsupported format. Please use txt or md.");
        return Ok(false);
    }

    let file_path = Path::new(file_path);
    if !file_path.exists() {
        print_warning(&format!("File does not exist: {}", file_path.display()));
        return Ok(false);
    }

    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            println!("Could not read file '{}': {}", file_path.display(), e);
            return Ok(false);
        },
    };

    if content.trim().is_empty() {
        print_warning(&format!("File is empty: {}", file_path.display()));
        return Ok(false);
    }

    let (title, content) = match format {
        "txt" => {
            parse_txt(&content)

        }
        "md" => {
            parse_md(&content)
        }
        // pattern will never be reached due to previous check
        _ => {
            print_warning("You are officially a wizard. This could should have been unreachable.");
            return Ok(false);
        }
    };

    let mut stmt = conn.prepare("SELECT COUNT(*) FROM notes WHERE title = ?1")
        .map_err(|e| NoteraError::Database(e))?;

    let count = stmt.query_row(params![title], |row| row.get(0)).unwrap_or(0);

    if count > 0 {
        print_warning(&format!("Note with title '{}' already exists. Skipping.", title));
        return Ok(false);
    }

    let timestamp = Local::now().format("%Y-%m-%d %H:%M").to_string();
    conn.execute(
        "INSERT INTO notes (title, content, timestamp) VALUES (?1, ?2, ?3)",
        params![title, content, timestamp],
    ).map_err(|e| NoteraError::Import(format!("Failed to import note into database: {}", e)))?;


    println!("✅ Imported note '{}' from '{}' successfully!", title, file_path.display());
    Ok(true)
}


pub fn import_dir(directory: &str) -> Result<()> {
    let conn = init_db()?;

    let dir_path = Path::new(directory);

    if !dir_path.exists() || !dir_path.is_dir() {
        println!("❌ Directory {} does not exist or is empty", dir_path.display());
        return Ok(());
    }

    let mut imported_count = 0;

    for entry in fs::read_dir(dir_path).map_err(|e| NoteraError::FileSystem(e, None))? {
        if let Ok(entry) = entry {
            let file_path = entry.path();
            if let Some(ext) = file_path.extension() {
                if ext == "txt" || ext == "md" {
                    let file_path_str = file_path.to_str().unwrap();
                    let format = ext.to_str().unwrap();

                    println!("🗄️ Importing file: {}", file_path_str);
                    if import_note(&conn, format, file_path_str)? {
                        imported_count += 1;
                    }
                }
            }
        }
    }

    println!();

    if imported_count > 0 {
        println!("✅ Successfully imported {} files", imported_count);
    } else {
        print_warning("No files were imported.");
    }

    Ok(())
}

fn parse_txt(content: &str) -> (String, String) {
    let mut lines = content.lines();
    let title = lines.next().unwrap_or("Untitled").trim().to_string();
    let content: String = lines.collect::<Vec<&str>>().join("\n").trim().to_string();

    (title, content)
}

fn parse_md(content: &str) -> (String, String) {
    let mut lines = content.lines();
    let title = lines.next().unwrap_or("Untitled").trim();

    let title = title.trim_start_matches('#').trim();

    let title = title.replace(" ", "_");

    let content: String = lines.collect::<Vec<&str>>().join("\n").trim().to_string();

    (title, content)
}