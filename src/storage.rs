use rusqlite::{params, Connection };
use std::{ fs, path::PathBuf };
use chrono::Local;
use crate::{
    error::{
        print_warning,
        with_path,
        NoteraError,
        Result,
    },
    config::Config,
    notes::Note,
};

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


pub fn save_note(note: &Note, conn: &Connection) -> Result<()> {

    conn.execute(
        "INSERT INTO notes (title, content, timestamp) VALUES (?1, ?2, ?3)",
        params![note.title.trim(), note.content, note.timestamp],
    ).map_err(|e| NoteraError::Database(e))?;

    println!("✅ Note saved successfully!");
    Ok(())
}

pub fn read_notes_from_db() -> Result<Vec<Note>> {

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

        notes.push(Note {
            title,
            content,
            timestamp,
        });
    }

    Ok(notes)
}

pub fn read_note(title: &str) -> Result<String> {
    let conn = init_db()?;

    let mut stmt = conn.prepare("SELECT title, content, timestamp FROM notes WHERE title = ?1")
        .map_err(|e| NoteraError::Database(e))?;

    let note_result = stmt.query_row([title], |row| {
        let title: String = row.get(0)?;
        let content: String = row.get(1)?;
        let timestamp: String = row.get(2)?;

        Ok((title, content, timestamp))
    }).map_err(|e| NoteraError::Database(e));

    let note = match &note_result {
        Ok(note) => note,
        Err(_) => {
            return Err(NoteraError::Other(format!("Note not found: '{}'", title)));
        }
    };


    let (title, content, timestamp) = note;

    Ok(format!("📝 Title: {}\n⏳ Created: {}\n\n{}", title, timestamp, content))
}

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

    let status = std::process::Command::new(editor)
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
                    fs::remove_file(entry.path()).unwrap_or_else(|err| {
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


pub fn rename_note(old_title_query: &str, new_title: &str) -> Result<()> {
    let conn = init_db()?;

    let mut stmt = conn.prepare("SELECT title FROM notes WHERE title LIKE ?")
        .map_err(|e| NoteraError::Database(e))?;

    let existing_titles: Vec<String> = stmt
        .query_map([format!("%{}%", old_title_query)],
        |row| row.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;

    match existing_titles.len() {
        0 => Err(NoteraError::Other(format!("No notes found with title '{}'", old_title_query))),
        1 => {
            let mut stmt = conn.prepare("SELECT COUNT(*) FROM notes WHERE title = ?")?;

            let count: i32 = stmt.query_row([new_title], |row| row.get(0))?;

            if count > 0 {
                return Err(NoteraError::Other(format!("Note with title '{}' already exists", new_title)));
            }

            conn.execute("UPDATE notes SET title = ? WHERE title = ?", [new_title, &existing_titles[0]])?;
            println!("✅ Successfully renamed note from '{}' to '{}'", old_title_query, new_title);

            Ok(())
        },
        _ => Err(NoteraError::Other(format!("Multiple notes found with title '{}'", old_title_query))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use tempfile::tempdir;

    fn setup_test_db() -> (Connection, tempfile::TempDir) {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = Connection::open(&db_path).unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL UNIQUE,
                content TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        ).unwrap();

        (conn, temp_dir)

    }

    #[test]
    fn test_save_and_read_note() {
        let (conn, _temp_dir) = setup_test_db();

        let title = "Test Note";
        let content = "This is a test note";

        let expected = ("Test Note".to_string(), "This is a test note".to_string());

        conn.execute(
            "INSERT INTO notes (title, content) VALUES (?1, ?2)",
            params![title, content],
        ).unwrap();

        let mut stmt = conn.prepare("SELECT title, content FROM notes WHERE title = ?1").unwrap();

        let note_result = stmt.query_row([title], |row| {
            let title: String = row.get(0).unwrap();
            let content: String = row.get(1).unwrap();
            Ok((title, content))
        }).unwrap();

        assert_eq!(note_result, expected);

    }


    #[test]
    fn test_delete_note() {
        let (conn, _temp_dir) = setup_test_db();

        // Setup - create a note
        let title = "Note to Delete";
        conn.execute(
            "INSERT INTO notes (title, content) VALUES (?1, ?2)",
            [title, "Content to delete"],
        ).unwrap();

        // Delete the note
        conn.execute("DELETE FROM notes WHERE title = ?1", [title]).unwrap();

        // Verify deletion
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM notes WHERE title = ?1").unwrap();
        let count: i64 = stmt.query_row([title], |row| row.get(0)).unwrap();

        assert_eq!(0, count);
    }


}

