
use chrono::Local;
use crate::{
    error::{
        with_path,
        NoteraError,
        Result,
    },
    config::Config
};

use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub title: String,
    pub content: String,
    pub timestamp: String,
}



impl Note {
    pub fn from_editor(title: &str) -> Result<Self> {
        let config = Config::load()?;
        let editor = &config.editor;

        let temp_dir = tempdir()?;
        let temp_file_path = temp_dir.path().join("notera_tmp_note.md");

        let status = std::process::Command::new(editor)
            .arg(&temp_file_path)
            .status()
            .map_err(|e| NoteraError::Other(format!("Failed to open editor: {}", e)))?;

        if !status.success() {
            return Err(NoteraError::Other("Editor exited with non-zero status".to_string()));
        }

        let content = fs::read_to_string(&temp_file_path)
            .map_err(|e| with_path(e, temp_file_path))?
            .trim()
            .to_string();

        if content.is_empty() {
            return Err(NoteraError::Other("Note discarded (empty content)".to_string()));
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M").to_string();

        Ok(Note {
            title: title.to_string(),
            content,
            timestamp,
        })
    }

    pub fn edit(&self) -> Result<Self> {
        let config = Config::load()?;
        let editor = &config.editor;



        let temp_dir = tempdir()?;
        let temp_file_path = temp_dir.path().join(format!("notera_{}.md", self.title.replace(" ", "_")));


        fs::write(&temp_file_path, &self.content)
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
            return Err(NoteraError::Other("Note discarded (empty content)".to_string()));
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M").to_string();

        Ok(Note {
            title: self.title.clone(),
            content: updated_content,
            timestamp,
        })
    }


    pub fn format(&self) -> String {
        format!("📝 Title: {}\n⏳ Created: {}\n\n{}\n", self.title, self.timestamp, self.content)
    }

    #[allow(dead_code)]
    pub fn print_raw(&self) {
        println!("{:#?}", self)
    }
}