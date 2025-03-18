use std::{ fs::{ self, File }, env, io::Read };
use reqwest::Client;
use serde_json::{ Value, json };
use crate::error::NoteraError;
use crate::storage;
use spinners::{ Spinner, Spinners };


const ENV_VAR: &str = "OPENAI_API_KEY";
const MODEL: &str = "gpt-4o-mini";
const URL: &str = "https://api.openai.com/v1/chat/completions";


pub async fn from_file(file_path: &str) -> Result<String, NoteraError> {
    let client = Client::new();

    let file_contents = fs::read_to_string(file_path)
        .map_err(|err| NoteraError::FileSystem(err, None))?;

    let openai_api_key: String = env::var(ENV_VAR)
        .unwrap_or_else(|_| {
            println!("Please set the {} environment variable", ENV_VAR);
            "No api key found".to_string()
        });

    let mut sp = Spinner::new(Spinners::Dots9, "Sending text file to ai (may take some time)".into());

    let body = json!({
        "model": MODEL,
        "messages": [
            {
                "role": "system",
                "content": "You are an ai bot that summarizes given text file content based on the content, assume that the person still needs to learn something, so don't over summarize."
            },
            {
                "role": "user",
                "content": format!("Summarize this lecture: {}", file_contents)
            }
        ]
    });

    let response = client.post(URL)
        .header("Authorization", format!("Bearer {}", openai_api_key))
        .json(&body)
        .send()
        .await.unwrap();

    sp.stop();

    let response_text = response.text().await.unwrap();

    let json: Value = serde_json::from_str(&response_text).unwrap();

    println!("\n");

    if let Some(completion) = json.get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
    {
        Ok(completion.to_string())
    } else {
        Err("unable to get completion back".into())
    }
}

pub async fn from_note(title: &str) -> Result<String, NoteraError> {

    let note_content = storage::read_note(title)?
        .join("\n\n");

    let client = Client::new();

    let openai_api_key: String = env::var(ENV_VAR)
        .unwrap_or_else(|_| {
            println!("Please set the {} environment variable", ENV_VAR);
            "No api key found".to_string()
        });

    let mut sp = Spinner::new(Spinners::Dots9, "Sending note to ai (may take some time)".into());

    let body = json!({
        "model": MODEL,
        "messages": [
            {
                "role": "system",
                "content": "You are an ai bot that summarizes given note based on the content, assume that the person still needs to learn something, so don't over summarize"
            },
            {
                "role": "user",
                "content": format!("Summarize this note: {}", note_content)
            }
        ]
    });

    let response = client.post(URL)
        .header("Authorization", format!("Bearer {}", openai_api_key))
        .json(&body)
        .send()
        .await
        .unwrap();

    sp.stop();

    let response_text = response.text().await.unwrap();

    let json: Value = serde_json::from_str(&response_text).unwrap();

    println!("\n");

    if let Some(completion) = json.get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
    {
        Ok(completion.to_string())
    } else {
        Err("unable to get completion back".into())
    }

}


pub async fn from_text(text: &str) -> Result<String, NoteraError> {
    let client = Client::new();

    let openai_api_key: String = env::var(ENV_VAR)
        .unwrap_or_else(|_| {
            println!("Please set the {} environment variable", ENV_VAR);
            "No api key found".to_string()
        });

    let body = json!({
        "model": MODEL,
        "messages": [
            {
                "role": "system",
                "content": "You are an ai bot that summarizes given note based on the content, assume that the person still needs to learn something, so don't over summarize"
            },
            {
                "role": "user",
                "content": format!("Summarize this note: {}", text)
            }
        ]
    });

    let mut sp = Spinner::new(Spinners::Dots9, "Sending note to ai (may take some time)".into());

    let response = client.post(URL)
        .header("Authorization", format!("Bearer {}", openai_api_key))
        .json(&body)
        .send()
        .await
        .unwrap();

    sp.stop();

    let response_text = response.text().await.unwrap();

    let json: Value = serde_json::from_str(&response_text).unwrap();

    println!("\n");

    if let Some(completion) = json.get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
    {
        Ok(completion.to_string())
    } else {
        Err("unable to get completion back".into())
    }
}


pub async fn transcribe_audio(audio_file: &str) -> Result<String, NoteraError> {
    let openai_api_key: String = env::var(ENV_VAR)
        .unwrap_or_else(|_| {
            println!("Please set the {} environment variable", ENV_VAR);
            "No api key found".to_string()
        });

    let client = Client::new();

    let mut sp = Spinner::new(Spinners::Dots9, "Sending audio file to ai for transcription (may take sometime depending on length)".into());

    let mut file = File::open(audio_file)?;
    let mut audio_data = Vec::new();
    file.read_to_end(&mut audio_data)?;

    let file_part = reqwest::multipart::Part::bytes(audio_data)
        .file_name("audio.mp3")
        .mime_str("audio/mpeg").unwrap();

    let form = reqwest::multipart::Form::new()
        .part("file", file_part)
        .text("model", "whisper-1");

    let response = client
        .post("https://api.openai.com/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", openai_api_key))
        .multipart(form)
        .send()
        .await.unwrap();

    sp.stop();

    println!("\n");

    let response_text = response.text().await.unwrap();

    let json: Value = serde_json::from_str(&response_text).unwrap();

    if let Some(transcript) = json.get("text") {
        Ok(transcript.as_str().unwrap().to_string())
    } else {
        Err("Failed to transcribe audio file".into())
    }
}


pub async fn transcribe_and_summarize(audio_file: &str) -> Result<String, NoteraError> {
    let transcript = transcribe_audio(audio_file).await?;
    let summary = from_text(&transcript).await?;
    Ok(summary)
}