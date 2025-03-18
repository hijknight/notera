use reqwest::Client;
use std::{ fs::File, io::Read, env };
use serde_json::Value;

const ENV_VAR: &str = "OPENAI_API_KEY";

pub async fn transcribe_audio(audio_path: &str) -> Result<String, Box<dyn std::error::Error>> {

    let openai_api_key: String = env::var(ENV_VAR)
        .unwrap_or_else(|_| {
            println!("Please set the {} environment variable", ENV_VAR);
            "No api key found".to_string()
        });

    let client = Client::new();

    let mut file = File::open(audio_path)?;
    let mut audio_data = Vec::new();
    file.read_to_end(&mut audio_data)?;

    let file_part = reqwest::multipart::Part::bytes(audio_data)
        .file_name("audio.mp3")
        .mime_str("audio/mpeg")?;

    let form = reqwest::multipart::Form::new()
        .part("file", file_part)
        .text("model", "whisper-1");

    let response = client
        .post("https://api.openai.com/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", openai_api_key))
        .multipart(form)
        .send()
        .await?;

    let response_text = response.text().await?;

    let json: Value = serde_json::from_str(&response_text)?;


    if let Some(transcript) = json.get("text") {
        Ok(transcript.as_str().unwrap().to_string())
    } else {
        Err("Failed to transcribe audio file".into())
    }
}







