use rusqlite::{ params, Connection };
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use crate::config::Config;
use chrono::Local;


pub fn get_db_path() -> PathBuf {
    let config = Config::load();
    let mut path = PathBuf::from(config.note_tmp_directory);
    fs::create_dir_all(&path).expect("❌ Failed to create note directory");
    path.push("notes.db");
    path
}

pub fn init_db() -> Connection {
    let db_path = get_db_path();
    let conn = Connection::open(db_path).expect("❌ Failed to open database");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            timestamp TEXT NOT NULL
        )",
        [],
    ).expect("❌ Failed to create table");

    conn
}


pub fn save_note(title: &str) {
    let conn = init_db();
    let editor = Config::load().editor;

    let temp_file_path = format!("/tmp/notera_{}.txt", title.replace(" ", "_"));

    // open temp file
    let _ = std::process::Command::new(&editor)
        .arg(&temp_file_path)
        .status()
        .expect("❌ Failed to open editor");

    // read note content
    let content = fs::read_to_string(&temp_file_path).unwrap_or_default().trim().to_string();
    if content.is_empty() {
        println!("⚠️ Note discarded (empty content).");
        return;
    }

    let timestamp = Local::now().format("%Y-%m-%d %H:%M").to_string();

    conn.execute(
        "INSERT INTO notes (title, content, timestamp) VALUES (?1, ?2, ?3)",
        params![title.trim(), content, timestamp],
    ).expect("❌ Failed to insert note");



    println!("✅ Note saved successfully!");
}


/// Read all existing notes
pub fn read_notes() -> Vec<String> {

    let conn = init_db();
    let mut stmt = conn.prepare("SELECT title, content, timestamp FROM notes ORDER BY timestamp DESC").unwrap();
    let notes_iter = stmt.query_map([], |row| {
        let title: String = row.get(0)?;
        let content: String = row.get(1)?;
        let timestamp: String = row.get(2).unwrap_or_else(|_| "Unknown Timestamp".to_string()); // Handle missing column

        Ok((title, content, timestamp))
    }).unwrap();


    let mut notes = Vec::new();

    for note in notes_iter {
        let (title, content, timestamp) = note.unwrap();
        notes.push(format!("📝 Title: {}\n⏳ Created: {}\n\n{}\n", title, timestamp, content));
    }
    notes
}
/// Edit an existing note
pub fn edit_note(title: &str) {
    let conn = init_db();
    let editor = Config::load().editor;

    let mut stmt = conn.prepare("SELECT content FROM notes WHERE title = ?1").unwrap();
    let content = stmt.query_row(params![title], |row| row.get(0)).unwrap_or_else(|_| "".to_string());

    if content.is_empty() {
        println!("⚠️ Note not found");
        return;
    }

    let temp_file_path = format!("/tmp/{}.txt", title.replace(" ", "_"));
    fs::write(&temp_file_path, &content).expect("❌ Failed to write file");

    let _ = std::process::Command::new(&editor)
        .arg(&temp_file_path)
        .status()
        .expect("❌ Failed to open editor");

    let updated_content = fs::read_to_string(&temp_file_path).unwrap_or_default().trim().to_string();

    if updated_content.is_empty() {
        println!("⚠️ No changes made.");
        return;
    }

    let timestamp = Local::now().format("%Y-%m-%d %H:%M").to_string();

    conn.execute(
        "UPDATE notes SET content = ?1, timestamp = ?2 WHERE title = ?3",
        params![updated_content, timestamp, title],
    ).expect("Failed to update note");

    println!("✅ Note updated successfully!")
}

pub fn delete_note(title: &str) {
    let conn = init_db();
    let title = title.trim();
    let result = conn.execute("DELETE FROM notes WHERE title = ?1", params![title]);
    // I forgot an s in notes, and it took me an hour to fix. ^^^ OMG

    match result {
        Ok(0) => println!("⚠️No note found with title '{}'", title),
        Ok(_) => {
            println!("✅ Note '{}' deleted", title);

            let temp_file_path = format!("/tmp/{}.txt", title.replace(" ", "_"));
            // if fs::remove_file(&temp_file_path).is_ok() {
            //     println!("Temp file '{}' deleted.", temp_file_path);
            // }
            match fs::remove_file(&temp_file_path) {
                Ok(_) => println!("✅ Temp file '{}' deleted.", temp_file_path),
                Err(e) => println!("❌ Failed to delete temp file: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to delete note: {}", e),
    }
}

// Clears all notes from db and tmp directory
pub fn clear_notes() {
    let conn = init_db();


    println!("⚠️ WARNING: This will permanently delete all notes. Type 'yes' to confirm");
    let mut confirmation = String::new();

    std::io::stdin()
        .read_line(&mut confirmation)
        .expect("Failed to read line");

    let confirmation = confirmation.trim().to_lowercase();

    if confirmation != "yes" {
        println!("❌ Clear operation aborted");
        return;
    }

    match conn.execute("DELETE FROM notes", params![]) {
        Ok(_) => println!("✅ Notes cleared from database."),
        Err(e) => {
            println!("❌ Failed to delete notes: {}", e);
            return;
        },
    }

    // delete temp files
    let temp_file_path = "/tmp/";

    if let Ok(entries) = fs::read_dir(temp_file_path) {
        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".txt") {
                    let _ = fs::remove_file(entry.path()).expect("❌Failed to delete temp file");
                }
            }
        }
    }

    println!("✅ Temporary note files cleared.");
}


pub fn export_notes(format: &str) {
    let conn = init_db();
    let mut stmt = conn.prepare("SELECT title, content, timestamp FROM notes ORDER BY timestamp DESC").unwrap();
    let export_path = Config::load().export_path;


    if !fs::exists(&export_path).unwrap() {
        fs::create_dir_all(&export_path).unwrap();
    }

    let notes_iter = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    }).unwrap();

    let mut notes = Vec::new();

    for note in notes_iter {
        let (title, content, timestamp) = note.unwrap();

        notes.push((title, content, timestamp));
    }

    if notes.is_empty() {
        println!("⚠️ No notes available to export_notes.");
        return;
    }

    let timestamp = Local::now().format("%Y-%m-%d_%H:%M").to_string();
    let default_filename = format!("notera-export_notes-{}.{}", timestamp, format);
    let output_path = format!("{}/{}", export_path, default_filename);

    let mut file = fs::File::create(&output_path).expect("❌ Failed to create file");

    match format {
        "txt" => {
            for (title, content, timestamp) in &notes {
                writeln!(file, "Title: {}\n\nCreated: {}\n-----\n\n{}\n-----------", title, timestamp, content).expect("❌ Failed to write to file");
            }
        },
        "md" => {
            for (title, content, timestamp) in &notes {
                writeln!(file, "## 📝 {}\n\n#### ⏳ *Created: {}*\n\n{}---\n", title, timestamp, content).expect("❌ Failed to write to file");
            }
        },
        _ => {
            println!("⚠️  Unsupported format: {}, use txt or md.", format);
            return;
        },
    }

    println!("✅  Notes exported successfully to {}", output_path);
}
