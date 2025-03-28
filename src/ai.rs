use crate::{error::{ NoteraError, Result }, storage};
use reqwest::Client;
use serde_json::{json, Value};
use spinners::{Spinner, Spinners};
use std::{
    env,
    fs::{self, File},
    io::Read,
};

const ENV_VAR: &str = "OPENAI_API_KEY";
const MODEL: &str = "gpt-4o-mini";
const URL: &str = "https://api.openai.com/v1/chat/completions";
const SPINNER_TYPE: Spinners = Spinners::BouncingBar;
#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    source: String,
    pub content: String,
}



impl Summary {
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

    pub async fn from_file(file_path: &str, prompt: Option<&str>) -> Result<Self> {
        let file_contents =
            fs::read_to_string(file_path).map_err(|err| NoteraError::FileSystem(err, None))?;

        let user_prompt: String = match prompt {
            Some(prompt) => format!("Here is the file's contents: {}.\n\n and here is the prompt given by the user: {}", file_contents, prompt),
            None => file_contents,
        };

        let ai_response = Self::prompt_ai(
            "file",
            "You are an ai bot that summarize a given text files contents. You should recognize the \
            file as being a transcript, maybe notes, or a list. FOr this function, the user has the ability to also \
            supply an optional prompt along with the file's contents. The prompt and the file's contents are clearly \
            specified as being two different things.",
            &user_prompt)
            .await?;

        Ok(ai_response)
    }

    pub async fn from_note(title: &str, prompt: Option<&str>) -> Result<Self> {
        let note_content = storage::read_note(title)?.join("\n\n");

        let user_prompt: String = match prompt {
            Some(prompt) => format!("Here is the note: {}.\n\n and here is the prompt given by the user: {}", note_content, prompt),
            None => note_content,
        };

        let ai_response = Self::prompt_ai(
            "note",
            "You are an ai bot that summarizes a given notes contents.\
             Making it look nicer, organizing, etc. Return markdown text without anything else unless the user is giving you a prompt. \
             Sometimes, the user will give you a prompt too in addition to the notes contents. \
             The prompt and the notes contents are clearly specified as being two different things.",
            &user_prompt
        ).await?;

        Ok(ai_response)
    }

    pub async fn from_prompt(prompt: &str) -> Result<Self> {

        let ai_response = Self::prompt_ai(
            "prompt",
            "You are an ai bot that a given prompt (or maybe a piece of text) by a user. \
            Try to figure out whether or not the user is giving you a piece of text to summarize, or just a normal chatgpt prompt. \
            If it is a normal chatgpt prompt, you shouldn't say anything that can be responded too, like 'how can i help you today'. \
            Because each prompt is independent.",
            prompt
        ).await?;

        Ok(ai_response)
    }

    // TODO: remove after project
    pub async fn from_interview(interview_audio: &str) -> Result<Self> {
        let transcript = Transcript::from_audio(interview_audio).await?;

        let ai_response = Self::prompt_ai(
            "interview",
            "You are an ai bot that is given an interview transcript, and you will see a very structured way of doing things, and all i need you to do is take the transcript,\
             and then put them in a the (somewhat) clearly defined separations. It is for a theology project, called the authentic happiness project. Here are the question that are asked: \
             What does the pursuit of happiness mean to you?,\
              What role does you faith in god play in your pursuit of happiness?, \
              What role does morality play in your pursuit of happiness?\
              Just take the interview transcript and put them in the correct sections.",
            &transcript.content
        ).await?;


        Ok(ai_response)
    }


    pub async fn from_audio(audio_file: &str, prompt: Option<&str>, local: &bool) -> Result<Self> {

        let transcript = if *local {
            Transcript::from_audio_local(audio_file).await?
        } else {
            Transcript::from_audio(audio_file).await?
        };

        let summary = Summary::from_transcript(&transcript, prompt).await?;

        Ok(summary)
    }

    pub fn print(&self) {
        println!("Summary source: {}\n", self.source);
        println!("{}", self.content);
    }

    async fn from_transcript(transcript: &Transcript, prompt: Option<&str>) -> Result<Self> {

        let user_prompt: String = match prompt {
            Some(prompt) => prompt.to_string(),
            None => transcript.content.clone(),
        };

        let ai_response = Self::prompt_ai(
            "transcript",
            "You are an ai bot that is given a transcript from a lecture or a presentation, \
            and then takes notes on it, as a college student would if they were sitting and taking the notes in the class. \
            However, if the transcription does not seem to have anything pertaining to a college lecture, just summarize it normally. \
            Only do this if you are 100% sure that the transcript is a lecture or presentation. Sometimes, the user will give you a prompt too in addition to the transcript.",
            &user_prompt
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

    pub async fn from_audio_local(audio_file: &str) -> Result<Self> {

        if !is_local_server_running().await {
            println!("\nNo local server running, to use `--local`, you must run your own python server!\n\
            You can find an example python server here: https://github.com/hijknight/notera/tree/ai-beta\n");

            return Err(NoteraError::Other("No local server running".to_string()));
        }

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
            .part("file", file_part);

        let response = client
            .post("http://localhost:5010/transcribe")
            .multipart(form)
            .send()
            .await
            .map_err(|err| NoteraError::Reqwest(err))?;


        sp.stop();

        let response_text = response
            .text()
            .await
            .map_err(|err| NoteraError::Other(format!("notera: error: {err}")))?;

        let json: Value = serde_json::from_str(&response_text).map_err(|err| NoteraError::SerdeJson(err))?;

        println!("\n");

        if let Some(transcript) = json.get("text") {
            Ok(Transcript {
                source: "local audio".to_string(),
                content: transcript.to_string(),
            })
        } else {
            Err(NoteraError::AI("Unable to find text in json".to_string()))
        }

    }


    pub async fn from_audio(audio_file: &str) -> Result<Self> {

        let metadata = fs::metadata(audio_file)
            .map_err(|err| NoteraError::FileSystem(err, None))?;

        const MAX_SIZE_BYTES: u64 = 25 * 1024 * 1024;

        if metadata.len() > MAX_SIZE_BYTES {
            return Err(NoteraError::AI(
                "Audio file exceeds 25MB limit. \
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
            Err(NoteraError::AI("If file is larger than 25mb, you need to run your own server locally".to_string()))
        }
    }

    pub fn print(&self) {
        println!("Transcript source: {}\n", self.source);
        println!("{}", self.content);
    }
}

async fn is_local_server_running() -> bool {
    let client = Client::new();

    let response = client
        .get("http://localhost:5010/transcribe")
        .send()
        .await;

    match response {
        Ok(_) => true,
        Err(_) => false,
    }
}
