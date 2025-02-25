use chrono::Local;
use std::fs::{OpenOptions, File};
use std::process::Command;
use std::env;
use std::io::{Write, Read};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Note {
    pub title: String,
    pub content: String,
    pub timestamp: String,  // store timestamp as a string
}

pub fn save_note(title: &str) {
    let temp_file_path = format!("/tmp/{}.txt", title.replace(" ", "_"));

    let _temp_file = File::create(&temp_file_path).expect("Failed to create temp file");
    let editor = env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
    let _ = Command::new(editor)
        .arg(&temp_file_path)
        .status()
        .expect("Failed to open editor");

    let mut note_content = String::new();
    File::open(&temp_file_path)
        .expect("Failed. to open the temp file after editing")
        .read_to_string(&mut note_content)
        .expect("Failed to read temp file");

    let new_note = Note {
        title: title.to_string(),
        content: note_content,
        timestamp: Local::now().format("&Y-%m-%d %H:%M:%S").to_string(),
    };

    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("notes.json")
        .expect("Unable to open file");

    let serialized = serde_json::to_string(&new_note).expect("Failed to serialize note");
    writeln!(&file, "{}", serialized).expect("Failed to write note");

    println!("Note saved successfully!");
}

pub fn read_notes() -> String {
    let mut file = File::open("notes.json").unwrap_or_else(|_| File::create("notes.json").unwrap());
    let mut content = String::new();
    file.read_to_string(&mut content).expect("Failed to read file");

    if content.trim().is_empty() {
        return "No notes available.".to_string();
    }

    let mut notes: Vec<Note> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<Note>(line).ok())
        .collect();

    // Sort notes by timestamp (newest first)
    notes.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let mut output = String::new();
    for note in notes {
        output.push_str(&format!("Title: {}\n{}\nCreated: {}\n\n", note.title, note.content, note.timestamp));
    }

    output
}


pub fn edit_note(title: &str) {
    let mut file = File::open("notes.json").unwrap_or_else(|_| File::create("notes.json").unwrap());
    let mut content = String::new();
    file.read_to_string(&mut content).expect("Failed to read file");

    let mut notes: Vec<Note> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<Note>(line).ok())
        .collect();

    if let Some(note) = notes.iter_mut().find(|note| note.title == title) {
        let temp_file_path = format!("/tmp/{}.txt", title.replace(" ", "_"));
        let mut temp_file = File::create(&temp_file_path).expect("Failed to create temp file");
        write!(temp_file, "{}", note.content).expect("Failed to write to temp file");

        let editor = env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
        let _ = Command::new(editor).
            arg(&temp_file_path).
            status().expect("Failed to open editor");

        let mut edited_content = String::new();
        File::open(&temp_file_path)
            .expect("Failed to open the temp file after editing")
            .read_to_string(&mut edited_content)
            .expect("Failed to read temp file");

        note.content = edited_content.trim().to_string();
        note.timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let new_content = notes.iter()
            .map(|note| serde_json::to_string(note).unwrap())
            .collect::<Vec<String>>()
            .join("\n");

        std::fs::write("notes.json", new_content).expect("Failed to write to file");

        println!("Note updated successfully!");
    } else {
        println!("Note with title '{}' not found.", title);
    }

    let new_content = notes.iter()
        .map(|note| serde_json::to_string(note).unwrap())
        .collect::<Vec<String>>()
        .join("\n");

    std::fs::write("notes.json", new_content).expect("Failed to write to file");
    println!("Note updated successfully!");
}


pub fn delete_note(title: &str) {
    let mut file = File::open("notes.json").unwrap_or_else(|_| File::create("notes.json").unwrap());
    let mut content = String::new();
    file.read_to_string(&mut content).expect("Failed to read file");

    let notes: Vec<Note> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<Note>(line).ok())
        .filter(|note| note.title != title)
        .collect();

    if notes.len() == content.lines().count() {
        println!("Note with title '{}' not found", title);
    }

    let new_content = notes.iter()
        .map(|note| serde_json::to_string(note).unwrap())
        .collect::<Vec<String>>()
        .join("\n");

    std::fs::write("notes.json", new_content).expect("Failed to write to file");
    println!("Note deleted successfully!");
}
