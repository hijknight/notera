use std::{
    path::Path,
    fs,
    io::Write,
};
use crate::{
    storage::init_db,
    error::{ print_warning, NoteraError },
    config::Config,
};
use chrono::Local;
use rusqlite::{params, Connection};


pub fn export_all() -> crate::error::Result<()> {
    let conn = init_db()?;
    let mut stmt = conn.prepare("SELECT title, content, timestamp FROM notes ORDER BY timestamp DESC")
        .map_err(|e| NoteraError::Database(e))?;
    let config = Config::load()?;
    let format = config.export_format.as_str();
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

pub fn export_note(title_query: &str) -> crate::error::Result<()> {
    let conn = init_db()?;

    let mut stmt = conn.prepare("SELECT title, content, timestamp FROM notes WHERE title = ?1").expect("❌ Note not found in database.");

    let config = Config::load()?;
    let format = config.export_format.as_str();
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

pub fn import_note(conn: &Connection, file_path: &str) -> crate::error::Result<bool> {

    let file_path = Path::new(file_path);
    if !file_path.exists() {
        print_warning(&format!("File does not exist: {}", file_path.display()));
        return Ok(false);
    }

    let format = file_path.extension().unwrap()
        .to_str().unwrap();

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


pub fn import_dir(directory: &str) -> crate::error::Result<()> {
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

                    println!("🗄️ Importing file: {}", file_path_str);
                    if import_note(&conn, file_path_str)? {
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