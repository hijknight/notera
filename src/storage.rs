use chrono::{Local, DateTime};
use std::fs::{OpenOptions, File};
use std::io::{Write, Read};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Note {
    pub title: String,
    pub content: String,
    pub timestamp: String,  // Store timestamp as a formatted string
}

pub fn save_note(title: &str, content: &str) {
    let now: DateTime<Local> = Local::now();
    let note = Note {
        title: title.to_string(),
        content: content.to_string(),
        timestamp: now.format("%Y-%m-%d %H:%M:%S").to_string(), // e.g., "2025-02-24 14:30:00"
    };

    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("notes.json")
        .expect("Unable to open file");

    let serialized = serde_json::to_string(&note).expect("Failed to serialize note");
    writeln!(&file, "{}", serialized).expect("Failed to write note");
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


pub fn edit_note(title: &str, new_content: &str) {
    let mut file = File::open("notes.json").unwrap_or_else(|_| File::create("notes.json").unwrap());
    let mut content = String::new();
    file.read_to_string(&mut content).expect("Failed to read file");

    let mut notes: Vec<Note> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<Note>(line).ok())
        .collect();

    if let Some(note) = notes.iter_mut().find(|note| note.title == title) {
        note.content = new_content.to_string();
    } else {
        println!("Note with title '{}' not found.", title);
        return;
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
