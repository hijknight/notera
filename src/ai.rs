use std::{ fs::{ self, File }, env, io::Read };
use reqwest::Client;
use serde_json::{ Value, json };
use crate::{ error::NoteraError, storage };
use spinners::{ Spinner, Spinners };

const ENV_VAR: &str = "OPENAI_API_KEY";
const MODEL: &str = "gpt-4o-mini";
const URL: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    source: String,
    pub content: String,
}

impl Summary {
    pub async fn from_file(file_path: &str) -> Result<Self, NoteraError> {
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
                "content": "You are an ai bot that summarizes a given text file content based on the content, assume that the person still needs to learn something, so don't over summarize."
            },
            {
                "role": "user",
                "content": format!("Summarize this text file: {}", file_contents)
            }
        ]
    });

        let response = client.post(URL)
            .header("Authorization", format!("Bearer {}", openai_api_key))
            .json(&body)
            .send()
            .await
            .map_err(|err| NoteraError::Reqwest(err))?;

        sp.stop();

        let response_text = response.text().await
            .map_err(|err| NoteraError::Reqwest(err))?;

        let json: Value = serde_json::from_str(&response_text).map_err(|err| NoteraError::SerdeJson(err))?;

        println!("\n");

        if let Some(completion) = json.get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
        {
            Ok(Summary {
                source: "file".to_string(),
                content: completion.to_string(),
            })
        } else {
            Err(NoteraError::AI("Unable to find choice message".to_string()))
        }
    }

    pub async fn from_note(title: &str) -> Result<Self, NoteraError> {

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
            .map_err(|err| NoteraError::Reqwest(err))?;

        sp.stop();

        let response_text = response.text().await
            .map_err(|err| NoteraError::Reqwest(err))?;

        let json: Value = serde_json::from_str(&response_text)
            .map_err(|err| NoteraError::SerdeJson(err))?;

        println!("\n");

        if let Some(completion) = json.get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
        {
            Ok(Summary {
                source: "note".to_string(),
                content: completion.to_string(),
            })
        } else {
            Err(NoteraError::AI("Unable to find choice message".to_string()))
        }

    }

    pub async fn from_text(text: &str) -> Result<Self, NoteraError> {
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
        ]});

        let mut sp = Spinner::new(Spinners::Dots9, "Sending text to ai (may take some time)".into());

        let response = client.post(URL)
            .header("Authorization", format!("Bearer {}", openai_api_key))
            .json(&body)
            .send()
            .await
            .map_err(|err| NoteraError::Reqwest(err))?;

        sp.stop();

        let response_text = response.text().await
            .map_err(|err| NoteraError::Reqwest(err))?;

        let json: Value = serde_json::from_str(&response_text)
            .map_err(|err| NoteraError::SerdeJson(err))?;

        println!("\n");

        if let Some(completion) = json.get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
        {
            Ok(Summary {
                source: "text".to_string(),
                content: completion.to_string(),
            })
        } else {
            Err(NoteraError::AI("Unable to find choice message".to_string()))
        }
    }

    async fn from_transcript(transcript: &Transcript) -> Result<Self, NoteraError> {
        let client = Client::new();

        let openai_api_key = env::var(ENV_VAR)
            .unwrap_or_else(|_| {
                println!("Please set the {} environment variable", ENV_VAR);
                "No api key found".to_string()
            });

        let body = json!({
        "model": MODEL,
        "messages": [
            {
                "role": "system",
                "content": "You are an ai bot that summarizes given transcript based on the content, assume that the person still needs to learn something from it, so don't over summarize"
            },
            {
                "role": "user",
                "content": format!("Summarize this transcript: {}", transcript.content)
            }
        ]});

        let mut sp = Spinner::new(Spinners::Dots9, "Sending transcript to ai (may take some time)".into());


        let response = client.post(URL)
            .header("Authorization", format!("Bearer {}", openai_api_key))
            .json(&body)
            .send()
            .await
            .map_err(|err| NoteraError::Reqwest(err))?;

        sp.stop();

        let response_text = response.text().await.map_err(|err| NoteraError::Reqwest(err))?;

        let json: Value = serde_json::from_str(&response_text)
            .map_err(|err| NoteraError::SerdeJson(err))?;

        println!("\n");

        if let Some(completion) = json.get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
        {
            Ok(Summary {
                source: "transcript".to_string(),
                content: completion.to_string(),
            })
        } else {
            Err(NoteraError::AI("Unable to find choice message".to_string()))
        }

    }

    pub async fn from_list_text(list_text: &str) -> Result<Self, NoteraError> {
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
                "content": "You are an ai bot is given todo list based on the content, helping to make the todo list nicer and formatted in raw markdown for someone to copy and paste into their file."
            },
            {
                "role": "user",
                "content": format!("Make my todo list nicer: {}", list_text)
            }
        ]});

        let mut sp = Spinner::new(Spinners::Dots9, "Sending text to ai (may take some time)".into());

        let response = client.post(URL)
            .header("Authorization", format!("Bearer {}", openai_api_key))
            .json(&body)
            .send()
            .await
            .map_err(|err| NoteraError::Reqwest(err))?;

        sp.stop();

        let response_text = response.text().await
            .map_err(|err| NoteraError::Reqwest(err))?;

        let json: Value = serde_json::from_str(&response_text)
            .map_err(|err| NoteraError::SerdeJson(err))?;

        println!("\n");

        if let Some(completion) = json.get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
        {
            Ok(Summary {
                source: "text".to_string(),
                content: completion.to_string(),
            })
        } else {
            Err(NoteraError::AI("Unable to find choice message".to_string()))
        }
    }

    pub async fn from_list_file(file_path: &str) -> Result<Self, NoteraError> {
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
                "content": "You are an ai bot is given todo list based on the content, helping to make the todo list nicer and formatted in raw markdown for someone to copy and paste into their file.",
            },
            {
                "role": "user",
                "content": format!("make this list nicer: {}", file_contents)
            }
        ]
    });

        let response = client.post(URL)
            .header("Authorization", format!("Bearer {}", openai_api_key))
            .json(&body)
            .send()
            .await
            .map_err(|err| NoteraError::Reqwest(err))?;

        sp.stop();

        let response_text = response.text().await
            .map_err(|err| NoteraError::Reqwest(err))?;

        let json: Value = serde_json::from_str(&response_text).map_err(|err| NoteraError::SerdeJson(err))?;

        println!("\n");

        if let Some(completion) = json.get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
        {
            Ok(Summary {
                source: "file".to_string(),
                content: completion.to_string(),
            })
        } else {
            Err(NoteraError::AI("Unable to find choice message".to_string()))
        }
    }


    pub async fn transcribe_and_summarize(audio_file: &str) -> Result<Self, NoteraError> {
        let transcript = Transcript::from_audio(audio_file).await?;
        let summary = Summary::from_transcript(&transcript).await?;
        Ok(summary)
    }

    pub fn print(&self) {
        println!("{}", self.content);
        println!("\n");
    }
}

#[derive(Debug, PartialEq)]
pub struct Transcript {
    pub source: String,
    pub content: String,
}

impl Transcript {
    pub async fn from_audio(audio_file: &str) -> Result<Self, NoteraError> {
        let openai_api_key: String = env::var(ENV_VAR)
            .unwrap_or_else(|_| {
                println!("Please set the {} environment variable", ENV_VAR);
                "No api key found".to_string()
            });

        let client = Client::new();

        let mut sp = Spinner::new(Spinners::Dots9, "Sending audio file to ai for transcription (may take sometime depending on length)".into());

        let mut file = File::open(audio_file)?;
        let mut audio_data = Vec::new();
        file.read_to_end(&mut audio_data).map_err(|err| NoteraError::FileSystem(err, None))?;

        let file_part = reqwest::multipart::Part::bytes(audio_data)
            .file_name("audio.mp3")
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

        println!("\n");

        let response_text = response.text().await
            .map_err(|err| NoteraError::Other(err.to_string()))?;

        let json: Value = serde_json::from_str(&response_text).map_err(|err| NoteraError::SerdeJson(err))?;

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
        println!("{}", self.content);
        println!("\n");
    }
}





