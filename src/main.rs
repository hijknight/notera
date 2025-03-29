/// handles all AI communications
mod ai;
/// deals with config functions
mod config;
/// has error handling enum
mod error;
/// deals with imports and exports
mod file_handling;
/// contains clean and init functions
mod setup;
/// has all database functions (save, edit, delete)
mod storage;
mod notes;

use clap::{CommandFactory, Parser, Subcommand};
use std::process;

/// `notera` CLI App
#[derive(Parser)]
#[command(name = "notera (AI-BETA)")]
#[command(version = "1.0.0-alpha")]
#[command(about = "A simple CLI-based note-taking app with powerful AI features.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, PartialEq)]
enum Commands {
    /// create a new file
    #[command(about = "create a new note with `notera new <TITLE>`", alias = "n")]
    New { title: String },

    /// view all notes or specify a title
    #[command(
        about = "view all notes with `notera view --all` or a specific notes `notera view --note <TITLE>`",
        alias = "l"
    )]
    View {
        #[arg(short, long, help = "view all notes")]
        all: bool,

        #[arg(long, help = "view a note with a specific title")]
        note: Option<String>,
    },

    /// edit a specific note
    #[command(about = "edit a specific note with `notera edit <TITLE>`", alias = "e")]
    Edit {
        title: String
    },

    /// rename a note
    #[command(about = "rename a specific note with `notera rename <OLD_TITLE> <NEW_TITLE>`", alias = "r")]
    Rename {
        old_title: String,
        new_title: String,
    },

    /// delete all or specific notes
    #[command(about = "delete a specific note with `notera delete <TITLE>`", alias = "d")]
    Delete {
        #[arg(short, long, help = "Delete a specific note with `notera delete <TITLE>`", value_name = "TITLE", alias = "d")]
        note: Option<String>,

        #[arg(short, long, help = "Delete all notes with `notera delete --all`")]
        all: bool,
    },

    /// import a directory of text files or a single file
    #[command(about = "import notes")]
    Import {
        #[arg(short, long, help = "Import all notes in a directory: <DIR_PATH>")]
        dir: Option<String>, // dir_path

        #[arg(short, long, value_name = "FILE_PATH", help = "Import a specific note file. Given a file path")]
        note: Option<String>,
    },

    /// export all current notes in database to a .txt or .md file or a specific note
    #[command(about = "export note(s)")]
    Export {
        #[arg(short, long, help = "Export all notes to a given format configured in config. Available formats: txt, md")]
        all: bool, // format

        #[arg(short, long, value_name = "TITLE", help = "Export a specific note. Provide format (.txt/.md) and note title")]
        note: Option<String>,
    },

    /// summarize a given notera note or a text file
    #[command(about = "summarize a given text file or note with ai")]
    Chat {

        #[arg(short='o', long, value_name = "PROMPT", help = "give notera a custom prompt, or add a description to something you are doing like `summarize --note <TITLE> --prompt <PROMPT>`")]
        prompt: Option<String>,

        #[arg(short, long, value_name = "FILE", help = "summarize a text file (.txt, .md)")]
        file: Option<String>,

        #[arg(short, long, value_name = "TITLE", help = "summarize a notera note given a title")]
        note: Option<String>,
        
        #[arg(short, long, help = "tell notera whether to export the summary to the notera/summaries folder or not")]
        export: bool,

        #[arg(short, long, help = "tell notera whether to print the summary to the terminal or not")]
        print: bool,
        
    },

    #[command(about = "summarize a given image file")]
    Image {
        #[arg(short, long, value_name = "IMAGE_FILE", help = "summarize an image file (.jpg, .png), optionally add a prompt")]
        file: Option<String>,
        
        #[arg(short='o', long, value_name = "PROMPT", help = "give notera a custom prompt, or add a description to something you are doing like `summarize --note <TITLE> --prompt <PROMPT>`")]
        prompt: Option<String>,

        #[arg(short, long, help = "tell notera whether to export the summary to the notera/summaries folder or not")]
        export: bool,

        #[arg(short, long, help = "tell notera whether to print the summary to the terminal or not")]
        print: bool,
    },

    /// transcribe a given audio file and export it notera/transcripts
    #[command(about = "transcribe an audio file with openai's whisper")]
    Audio {

        #[arg(short, long, value_name = "AUDIO_FILE_PATH", help = "summarize an audio file (.m4a, .mp3, .wav)")]
        summarize: Option<String>,
        
        #[arg(short, long, value_name = "AUDIO_FILE_PATH", help = "transcribe a given audio file (Only .m4a - Voice memos, quick time player)")]
        transcribe: Option<String>,

        #[arg(short='o', long, value_name = "PROMPT", help = "give notera a custom prompt, or add a description to something you are doing like `audio --summarize <AUDIO_FILE_PATH> --prompt <PROMPT>`")]
        prompt: Option<String>,

        #[arg(short, long, help = "tell notera whether to export the transcript to the notera/transcripts folder or not")]
        export: bool,

        #[arg(short, long, help = "tell notera whether to print the transcript to the terminal or not")]
        print: bool,

    },

    /// change config
    #[command(about = "open the notera config file with `notera config`")]
    Config,

    /// initialize or reinitialize `notera` for first use
    #[command(about = "initialize notera with `notera init`")]
    Init,

    /// clean all notera files (optionally the Documents/notera folder)
    #[command(about = "DANGER: clean notera's data with `notera clean`")]
    Clean,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if !setup::is_initialized() && cli.command != Some(Commands::Init) {
        println!("❌ Notera not yet setup. Run `notera init` to initialize and set configuration options.");
        process::exit(1);
    }

    let result = match &cli.command {
        Some(Commands::New { title }) => {
            let conn = storage::init_db().unwrap();
            let note = notes::Note::from_editor(title).unwrap();
            storage::save_note(&note, &conn)
        },

        Some(Commands::View { all, note }) => {


            if *all {
                match storage::read_notes_from_db() {
                    Ok(notes) => {
                        let mut formatted_notes = Vec::new();

                        for note in notes {
                            formatted_notes.push(note.format());
                        };
                        println!("{}", formatted_notes.join("\n\n"));
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            } else if let Some(note) = note {
                match storage::read_note(note) {
                    Ok(note) => {
                        println!("{}", note);
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            } else {
                println!("No valid import option provided. Use `--all` or `--note`.");
                Ok(())
            }
        }

        Some(Commands::Edit { title }) => storage::edit_note(title),

        Some(Commands::Rename {
            old_title,
            new_title,
        }) => storage::rename_note(old_title, new_title),

        Some(Commands::Delete { note, all }) => {
            if *all {
                storage::clear_notes()
            } else if let Some(args) = note {
                let note_title = args;
                storage::delete_note(note_title)
            } else {
                println!("No valid delete option provided. Use `--all` or `--note`.");
                Ok(())
            }
        }

        Some(Commands::Import { dir, note }) => {
            if let Some(directory_given) = dir {
                file_handling::import_dir(directory_given)
            } else if let Some(note_file_path) = note {
                match storage::init_db() {
                    Ok(conn) => {
                        file_handling::import_note(&conn, note_file_path).unwrap_or_else(|err| {
                            println!("error importing file {}", err);
                            false
                        });
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            } else {
                println!("❌ No valid import option provided. Use `--dir` or `--note`.");
                Ok(())
            }
        }

        Some(Commands::Export { all, note }) => {
            if *all {
                file_handling::export_all()
            } else if let Some(args) = note {
                let title = args;
                file_handling::export_note(title)
            } else {
                println!("❌ No valid export option provided. Use `--all` or `--note`.");
                Ok(())
            }
        }

        Some(Commands::Chat {
            file,
            note,

            print,
            export,
            
            prompt,
        }) => {
            if let Some(file) = file {
                if !*print && !*export {
                    println!("No flags given for summary to print or export. Use `--print` and/or `--export` to print or export the summary.");
                } else {
                    let summary_result = match prompt {
                        Some(prompt) => ai::Completion::from_file(file, Some(prompt)),
                        None => ai::Completion::from_file(file, None),
                    }.await;

                    match summary_result {
                        Ok(summary) => {
                            if *print {
                                summary.print();
                            }

                            if *export {
                                if let Ok(_) = file_handling::export_completion(&summary) {
                                    println!(
                                        "Completion exported to {}/summaries",
                                        config::Config::load().unwrap().notera_files_path
                                    );
                                } else {
                                    println!(
                                        "Unable to export summary. Please check your permissions."
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                }

                Ok(())
            } else if let Some(note) = note {
                if !*print && !*export {
                    println!("No flag given for summary to print or export. Use `--print` and/or `--export` to print or export the summary.");
                } else {
                    let summary_result = match prompt {
                        Some(prompt) => ai::Completion::from_note(note, Some(prompt)),
                        None => ai::Completion::from_note(note, None),
                    }.await;

                    match summary_result {
                        Ok(summary) => {
                            if *print {
                                summary.print();
                            }

                            if *export {
                                if let Ok(_) = file_handling::export_completion(&summary) {
                                    println!(
                                        "Completion exported to {}/summaries",
                                        config::Config::load().unwrap().notera_files_path
                                    );
                                } else {
                                    println!(
                                        "Unable to export summary. Please check your permissions."
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                }

                Ok(())
            } else if let Some(prompt) = prompt {
                if !*print && !*export {
                    println!("No flag given for summary to print or export. Use `--print` and/or `--export` to print or export the summary.");
                } else {
                    let summary_result = ai::Completion::from_prompt(prompt).await;

                    match summary_result {
                        Ok(summary) => {
                            if *print {
                                summary.print();
                            }

                            if *export {
                                if let Ok(_) = file_handling::export_completion(&summary) {
                                    println!(
                                        "✅ Completion exported to {}/summaries",
                                        config::Config::load().unwrap().notera_files_path
                                    );
                                } else {
                                    println!(
                                        "⚠️ Unable to export summary. Please check your permissions."
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                }

                Ok(())
            } else {
                println!("⚠️ No valid import option provided. Use `--file`, `--note`, `--prompt`.");
                Ok(())
            }
        }

        Some(Commands::Audio { summarize, transcribe, print , export , prompt}) => {
            if let Some(audio) = transcribe {
                if !*print && !*export {
                    println!("⚠️ No flag given for summary to print or export. Use `--print` and/or `--export` to print or export the summary.");
                } else {
                    let transcription_result = ai::Transcript::transcribe(audio).await;

                    match transcription_result {
                        Ok(transcript) => {
                            if *print {
                                transcript.print();
                            }

                            if *export {
                                if let Ok(_) = file_handling::export_transcript(&transcript) {
                                    println!(
                                        "Transcript exported to {}/transcripts",
                                        config::Config::load().unwrap().notera_files_path
                                    );
                                } else {
                                    println!(
                                        "⚠️ Unable to export transcript. Please check your permissions."
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                }
                Ok(())
            } else if let Some(audio) = summarize {
                if !*print && *export {
                    println!("⚠️ No flag given for summary to print or export. Use `--print` and/or `--export` to print or export the summary.");
                } else {
                    let completion = match prompt {
                        Some(prompt) => ai::Completion::from_audio(audio, Some(prompt)),
                        None => ai::Completion::from_audio(audio, None),
                    }.await;
                    
                    match completion {
                        Ok(summary) => {
                            if *print {
                                summary.print();
                            }
                            
                            if *export {
                                if let Ok(_) = file_handling::export_completion(&summary) {
                                    println!(
                                        "Completion exported to {}/summaries",
                                        config::Config::load().unwrap().notera_files_path
                                    );
                                } else {
                                    println!(
                                        "⚠️ Unable to export transcript. Please check your permissions."
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                    
                }
                Ok(())
            } else {
                println!("⚠️ No valid import option provided. Use `--audio`.");
                Ok(())
            }
        }
        
        Some(Commands::Image { file, prompt, export, print }) => {
            if let Some(file) = file {
                if !*print && !*export {
                    println!("⚠️ No flag given for summary to print or export. Use `--print` and/or `--export` to print or export the summary.");
                } else {
                    let completion = match prompt {
                        Some(prompt) => ai::Completion::from_image("image", file, Some(prompt)),
                        None => ai::Completion::from_image("image", file, None),
                    }.await;
                    
                    match completion {
                        Ok(completion) => {
                            if *print {
                                completion.print();
                            }

                            if *export {
                                if let Ok(_) = file_handling::export_completion(&completion) {
                                    println!(
                                        "Completion completion exported to {}/transcripts",
                                        config::Config::load().unwrap().notera_files_path
                                    );
                                } else {
                                    println!(
                                        "⚠️ Unable to export chat completion. Please check your permissions."
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                }
            }
            
            Ok(())
        }

        Some(Commands::Config) => setup::open_config(),

        Some(Commands::Init) => setup::init(),

        Some(Commands::Clean) => setup::clean(),

        None => Cli::command().print_help().map_err(|e| e.into()),
    };

    if let Err(error) = result {
        error::handle_error(&error);
    }
}
