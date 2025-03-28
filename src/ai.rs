use crate::{error::{ NoteraError, Result }, storage};
use reqwest::Client;
use serde_json::{ json, Value };
use spinners::{ Spinner, Spinners };
use std::{
    env,
    fs::{self, File},
    io::Read,
    path::Path,
};
use base64::{Engine as _, engine};
const ENV_VAR: &str = "OPENAI_API_KEY";
const CHAT_MODEL: &str = "gpt-4o-mini";
const GPT_URL: &str = "https://api.openai.com/v1/chat/completions";
const WHISPER_URL: &str = "https://api.openai.com/v1/audio/transcriptions";
const SPINNER_TYPE: Spinners = Spinners::BouncingBar;

#[derive(Debug, Clone, PartialEq)]
pub struct Completion {
    source: String,
    pub content: String,
}

impl Completion {


    async fn prompt_ai(source: &str, system_prompt: &str, user_prompt: &str) -> Result<Self> {
        let client = Client::new();

        let openai_api_key: String = env::var(ENV_VAR).unwrap_or_else(|_| {
            println!("Please set the {} environment variable", ENV_VAR);
            "No api key found".to_string()
        });

        let mut sp = Spinner::new(
            SPINNER_TYPE,
            "Working with ai...".into(),
        );

        let body = json!({
            "model": CHAT_MODEL,
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
            .post(GPT_URL)
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
            Ok(Completion {
                source: source.to_string(),
                content: completion.to_string(),
            })
        } else {
            Err(NoteraError::AI("Unable to find choice message".to_string()))
        }
    }

    pub async fn from_image(source: &str, image_path: &str, prompt: Option<&str>) -> Result<Self> {
        let file_path = Path::new(image_path);

        let user_prompt: String = match prompt {
            Some(prompt) => format!(
                "Here is an image. Please use the prompt to help you respond accurately: \"{}\"",
                prompt
            ),
            None => "Please summarize the attached image.".to_string(),
        };


        if !file_path.exists() {
            return Err(NoteraError::Other(format!("File {} does not exist", image_path)));
        }

        let openai_api_key = env::var(ENV_VAR).unwrap_or_else(|_| {
            println!("Please set the {} environment variable", ENV_VAR);
            "No api key found".to_string()
        });

        let client = Client::new();

        let mut sp = Spinner::new(
            SPINNER_TYPE,
            "Working with ai...".into(),
        );

        let image_data = fs::read(image_path)
            .map_err(|err| NoteraError::FileSystem(err, None))?;

        let base64_image = engine::general_purpose::STANDARD.encode(&image_data);


        let mime_type = match file_path.extension().and_then(|ext| ext.to_str()) {
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("webp") => "image/webp",
            Some("gif") => "image/gif",
            Some("bmp") => "image/bmp",
            Some("tiff") => "image/tiff",
            Some("svg") => "image/svg+xml",
            Some("ico") => "image/x-icon",
            Some("psd") => "image/vnd.adobe.photoshop",
            Some("eps") => "image/vnd.adobe.postscript",
            Some("ai") => "image/vnd.adobe.illustrator",
            Some("raw") => "image/x-raw",
            Some("heic") => "image/heic",
            Some("heif") => "image/heif",
            Some("indd") => "image/vnd.dece.graphic",
            Some("indt") => "image/vnd.dece.tilemap",
            Some("indp") => "image/vnd.dece.picture",
            _ => "image/jpeg",  // default to JPEG if unknown
        };


        let body = json!({
            "model": CHAT_MODEL,
            "messages": [
                {
                    "role": "system",
                    "content": "You are an ai bot that summarizes an image. and is sometime given a prompt."
                },
                {
                    "role": "user",
                    "content": [
                    {
                        "type": "text",
                        "text": user_prompt
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:{};base64,{}", mime_type, base64_image)
                        }
                    }
                ]
                }
            ]
        });

        let response = client
            .post(GPT_URL)
            .header("Authorization", format!("Bearer {}", openai_api_key))
            .json(&body)
            .send().await
            .map_err(|err| NoteraError::Reqwest(err))?;


        let response_text = response
            .text().await
            .map_err(|err| NoteraError::Reqwest(err))?;

        sp.stop();

        println!("\n");

        let json: Value = serde_json::from_str(&response_text).map_err(|err| NoteraError::SerdeJson(err))?;


        if let Some(completion) = json
            .get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
        {
            Ok(Completion {
                source: source.to_string(),
                content: completion.to_string(),
            })
        } else {
            Err(NoteraError::AI("Unable to find choice message".to_string()))
        }
    }

    pub fn print(&self) {
        println!("Transcript source: {}\n", self.source);
        println!("{}", self.content);
    }

    pub async fn from_file(file_path: &str, prompt: Option<&str>) -> Result<Self> {
        let file_contents =
            fs::read_to_string(file_path).map_err(|err| NoteraError::FileSystem(err, None))?;

        let system_prompt = "You are an ai bot that summarize a given text files contents. You should recognize the \
            file as being a transcript, maybe notes, or a list. FOr this function, the user has the ability to also \
            supply an optional prompt along with the file's contents. The prompt and the file's contents are clearly \
            specified as being two different things.";

        let user_prompt: String = match prompt {
            Some(prompt) => format!("Here is the file's contents: {}.\n\n and here is the prompt given by the user: {}", file_contents, prompt),
            None => file_contents,
        };

        let ai_response = Self::prompt_ai(
            "file",
            system_prompt,
            &user_prompt,
        ).await?;

        Ok(ai_response)
    }

    pub async fn from_note(title: &str, prompt: Option<&str>) -> Result<Self> {
        let note_content = match storage::read_note(title) {
            Ok(note) => format!("{}", note),
            Err(e) => return Err(e),
        };

        let user_prompt: String = match prompt {
            Some(prompt) => format!("Here is the note: {}.\n\n and here is the prompt given by the user: {}", note_content, prompt),
            None => note_content,
        };

        let system_prompt: &str = "You are an ai bot that summarizes a given notes contents.\
             Making it look nicer, organizing, etc. Return markdown text without anything else unless the user is giving you a prompt. \
             Sometimes, the user will give you a prompt too in addition to the notes contents. \
             The prompt and the notes contents are clearly specified as being two different things.";

        let ai_response = Self::prompt_ai(
            "note",
            system_prompt,
            &user_prompt,
        ).await?;

        Ok(ai_response)
    }

    pub async fn from_prompt(prompt: &str) -> Result<Self> {

        let system_prompt: &str = "You are an ai bot that a given prompt (or maybe a piece of text) by a user. \
            Try to figure out whether or not the user is giving you a piece of text to summarize, or just a normal chatgpt prompt. \
            If it is a normal chatgpt prompt, you shouldn't say anything that can be responded too, like 'how can i help you today'. \
            Because each prompt is independent.";

        let ai_response = Self::prompt_ai(
            "prompt",
            system_prompt,
            prompt,
        ).await?;

        Ok(ai_response)
    }

    // TODO: remove after projec
    pub async fn from_audio(audio_file: &str, prompt: Option<&str>) -> Result<Self> {

        let transcript = Transcript::transcribe(audio_file).await?;

        let summary = Completion::from_transcript(&transcript, prompt).await?;

        Ok(summary)
    }

    async fn from_transcript(transcript: &Transcript, prompt: Option<&str>) -> Result<Self> {

        let user_prompt: String = match prompt {
            Some(prompt) => format!("\n\n Here is the prompt given by the user: {}, Here is the transcript: {}.", prompt, transcript.content),
            None => transcript.content.clone(),
        };

        let system_prompt: &str = "You are an ai bot that is given a transcript from a lecture or a presentation, \
            and then takes notes on it, as a college student would if they were sitting and taking the notes in the class. \
            However, if the transcription does not seem to have anything pertaining to a college lecture, just summarize it normally. \
            Only do this if you are 100% sure that the transcript is a lecture or presentation. \
            Sometimes, the user will give you a prompt too in addition to the transcript.";

        let ai_response = Self::prompt_ai(
            "transcript",
            system_prompt,
            &user_prompt,
        ).await?;

        Ok(ai_response)
    }
}

#[derive(Debug, PartialEq)]
pub struct Transcript {
    pub source: String,
    pub content: String,
}

impl Transcript {
    pub async fn transcribe(audio_file: &str) -> Result<Self> {

        let metadata = fs::metadata(audio_file)
            .map_err(|err| NoteraError::FileSystem(err, None))?;

        const MAX_SIZE_BYTES: u64 = 25 * 1024 * 1024;

        if metadata.len() > MAX_SIZE_BYTES {
            return Err(NoteraError::AI(
                "Transcript file exceeds 25MB limit. \
                Please use a smaller file or run your local server".to_string()
            ));
        }

        let openai_api_key: String = env::var(ENV_VAR).unwrap_or_else(|_| {
            println!("Please set the {} environment variable", ENV_VAR);
            "No api key found".to_string()
        });

        let client = Client::new();

        let mut sp = Spinner::new(
            SPINNER_TYPE,
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
            .post(WHISPER_URL)
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
            Err(NoteraError::AI("If file is larger than 25mb, you need to run your own server locally".to_string()))
        }
    }

    pub fn print(&self) {
        println!("Transcript source: {}\n", self.source);
        println!("{}", self.content);
    }
}



