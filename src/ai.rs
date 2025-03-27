use crate::{error::{ NoteraError, Result }, storage};
use reqwest::Client;
use serde_json::{json, Value};
use spinners::{Spinner, Spinners};
use std::{
    env,
    fs::{self, File},
    io::Read,
    // time::Duration,
    // thread::sleep,
};


const ENV_VAR: &str = "OPENAI_API_KEY";
const MODEL: &str = "gpt-4o-mini";
const URL: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    source: String,
    pub content: String,
}
// TODO:
// fn start_fun_spinner(spinner_type: Spinners) {
//     let mut sp1 = Spinner::new(spinner_type.clone(), "Contacting AI...".into());
//     sleep(Duration::from_millis(500));
//     sp1.stop();
//
//     let mut sp2 = Spinner::new(spinner_type.clone(), "Processing new info...".into());
//     sleep(Duration::from_millis(300));
//     sp2.stop();
// }

impl Summary {
    async fn prompt_ai(source: &str, system_prompt: &str, user_prompt: &str) -> Result<Self> {
        let client = Client::new();

        let openai_api_key: String = env::var(ENV_VAR).unwrap_or_else(|_| {
            println!("Please set the {} environment variable", ENV_VAR);
            "No api key found".to_string()
        });

        let mut sp = Spinner::new(
            Spinners::Dots9,
            "Talking with ai...".into(),
        );

        let body = json!({
            "model": MODEL,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": user_prompt
                }
            ]
        });

        let response = client
            .post(URL)
            .header("Authorization", format!("Bearer {}", openai_api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| NoteraError::Reqwest(e))?;

        sp.stop();

        let response_text = response
            .text()
            .await
            .map_err(|e| NoteraError::Reqwest(e))?;

        let json: Value =
            serde_json::from_str(&response_text).map_err(|e| NoteraError::SerdeJson(e))?;

        println!("\n");

        if let Some(completion) = json
            .get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
        {
            Ok(Summary {
                source: source.to_string(),
                content: completion.to_string(),
            })
        } else {
            Err(NoteraError::AI("Unable to find choice message".to_string()))
        }

    }

    pub async fn from_file(file_path: &str) -> Result<Self> {
        let file_contents =
            fs::read_to_string(file_path).map_err(|err| NoteraError::FileSystem(err, None))?;

        let ai_response = Self::prompt_ai(
            "file",
            "You are an ai bot that summarize a given text files contents",
            &file_contents)
            .await?;

        Ok(ai_response)
    }

    pub async fn from_note(title: &str) -> Result<Self> {
        let note_content = storage::read_note(title)?.join("\n\n");

        let ai_response = Self::prompt_ai(
            "note",
            "You are an ai bot that summarizes a given notes contents. Making it look nicer, organizing, etc. Return markdown text without anything else",
            &note_content
        ).await?;

        Ok(ai_response)
    }

    pub async fn from_text(text: &str) -> Result<Self> {

        let ai_response = Self::prompt_ai(
            "text",
            "You are an ai bot that a given prompt (or maybe a piece of text) by a user. Try to figure out whether or not the user is giving you a piece of text to summarize, or just a normal chatgpt prompt. If it is a normal chatgpt prompt, you shouldn't say anything that can be responded too, like 'how can i help you today'. Because each prompt is independent.",
            text
        ).await?;

        Ok(ai_response)
    }


    pub async fn from_list_file(file_path: &str) -> Result<Self> {

        let file_contents = fs::read_to_string(file_path).map_err(|err| NoteraError::FileSystem(err, None))?;

        let ai_response = Self::prompt_ai(
            "list",
            "You are an ai bot that is given a list of something; could be a todo list, grocery list, or a list that could be in someones planner. The user will expect a nicer, more organized version of the list back (in markdown) with all of the information retained",
            &file_contents
        ).await?;

        Ok(ai_response)

    }

    async fn from_transcript(transcript: &Transcript) -> Result<Self> {

        let ai_response = Self::prompt_ai(
            "transcript",
            "You are an ai bot that is given a transcript from a lecture or a presentation, and then takes notes on it, as a college student would if they were sitting and taking the notes in the class. However, if the transcription does not seem to have anything pertaining to a college lecture, just summarize it normally. Only do this if you are 100% sure that the transcript is a lecture or presentation.",
            &transcript.content
        ).await?;

        Ok(ai_response)
    }


    pub async fn from_audio(audio_file: &str) -> Result<Self> {
        let transcript = Transcript::from_audio(audio_file).await?;
        let summary = Summary::from_transcript(&transcript).await?;
        Ok(summary)
    }

    pub fn print(&self) {
        println!("Summary source: {}\n", self.source);
        println!("{}", self.content);
    }
}

#[derive(Debug, PartialEq)]
pub struct Transcript {
    pub source: String,
    pub content: String,
}

impl Transcript {
    pub async fn from_audio(audio_file: &str) -> Result<Self> {
        let openai_api_key: String = env::var(ENV_VAR).unwrap_or_else(|_| {
            println!("Please set the {} environment variable", ENV_VAR);
            "No api key found".to_string()
        });

        let client = Client::new();

        let mut sp = Spinner::new(
            Spinners::Dots9,
            "Transcribing... (time dependent on length)".into(),
        );

        let mut file = File::open(audio_file)?;
        let mut audio_data = Vec::new();
        file.read_to_end(&mut audio_data)
            .map_err(|err| NoteraError::FileSystem(err, None))?;

        let file_part = reqwest::multipart::Part::bytes(audio_data)
            .file_name("audio.m4a")
            .mime_str("audio/mpeg")
            .map_err(|err| NoteraError::Other(err.to_string()))?;

        let form = reqwest::multipart::Form::new()
            .part("file", file_part)
            .text("model", "whisper-1");

        let response = client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", openai_api_key))
            .multipart(form)
            .send()
            .await
            .map_err(|err| NoteraError::Reqwest(err))?;

        sp.stop();

        let response_text = response
            .text()
            .await
            .map_err(|err| NoteraError::Other(format!("notera: error: {err}")))?;

        let json: Value =
            serde_json::from_str(&response_text).map_err(|err| NoteraError::SerdeJson(err))?;

        println!("\n");

        if let Some(transcript) = json.get("text") {
            #[allow(dead_code)]
            Ok(Transcript {
                source: "audio".to_string(),
                content: transcript.to_string(),
            })
        } else {
            Err(NoteraError::AI("Unable to find choice message".to_string()))
        }
    }

    pub fn print(&self) {
        println!("Transcript source: {}\n", self.source);
        println!("{}", self.content);
    }
}
