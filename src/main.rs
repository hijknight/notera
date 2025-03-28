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
    Summarize {

        #[arg(short='o', long, value_name = "PROMPT", help = "give notera a custom prompt, or add a description to something you are doing like `summarize --note <TITLE> --prompt <PROMPT>`")]
        prompt: Option<String>,

        #[arg(short, long, value_name = "FILE", help = "summarize a text file (.txt, .md)")]
        file: Option<String>,

        #[arg(short, long, value_name = "TITLE", help = "summarize a notera note given a title")]
        note: Option<String>,

        #[arg(short, long, value_name = "AUDIO_FILE", help = "tell notera whether to print the summary to the terminal or not")]
        audio: Option<String>,

        // TODO: remove after project
        #[arg(long, value_name = "AUDIO_FILE", help = "for authentic happiness project")]
        interview: Option<String>,

        #[arg(short='L', long, help = "specify whether or not to transcribe the file locally or not. MUST RUN OWN SERVER")]
        local: bool,

        #[arg(short, long, help = "tell notera whether to export the summary to the notera/summaries folder or not")]
        export: bool,

        #[arg(short, long, help = "tell notera whether to print the summary to the terminal or not")]
        print: bool,
    },

    /// transcribe a given audio file and export it notera/transcripts
    #[command(about = "transcribe an audio file with openai's whisper")]
    Transcribe {

        #[arg(short, long, value_name = "AUDIO_FILE_PATH", help = "transcribe a given audio file (Only .m4a - Voice memos, quick time player)")]
        audio: Option<String>,

        #[arg(short, long, help = "tell notera whether to export the transcript to the notera/transcripts folder or not")]
        export: bool,

        #[arg(short, long, help = "tell notera whether to print the transcript to the terminal or not")]
        print: bool,

        #[arg(short, long, help = "DEV AND GITHUB ONLY: specify if you want to transcribe the given audio file locally")]
        local: bool,

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
        Some(Commands::New { title }) => storage::save_note(title),

        Some(Commands::View { all, note }) => {
            if *all {
                match storage::read_notes() {
                    Ok(notes) => {
                        println!("{}", notes.join("\n\n"));
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            } else if let Some(note) = note {
                match storage::read_note(note) {
                    Ok(notes) => {
                        println!("{}", notes.join("\n\n"));
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

        Some(Commands::Summarize {
            file,
            note,
            print,
            export,
            audio,
            interview,
            prompt,
            local,
        }) => {
            if let Some(file) = file {
                if !*print && !*export {
                    println!("No flags given for summary to print or export. Use `--print` and/or `--export` to print or export the summary.");
                } else {
                    let summary_result = match prompt {
                        Some(prompt) => ai::Summary::from_file(file, Some(prompt)),
                        None => ai::Summary::from_file(file, None),
                    }.await;

                    match summary_result {
                        Ok(summary) => {
                            if *print {
                                summary.print();
                            }

                            if *export {
                                if let Ok(_) = file_handling::export_summary(&summary) {
                                    println!(
                                        "Summary exported to {}/summaries",
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
                        Some(prompt) => ai::Summary::from_note(note, Some(prompt)),
                        None => ai::Summary::from_note(note, None),
                    }.await;

                    match summary_result {
                        Ok(summary) => {
                            if *print {
                                summary.print();
                            }

                            if *export {
                                if let Ok(_) = file_handling::export_summary(&summary) {
                                    println!(
                                        "Summary exported to {}/summaries",
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
            } else if let Some(audio) = audio {
                if !*print && !*export {
                    println!("No flag given for summary to print or export. Use `--print` and/or `--export` to print or export the summary.");
                } else {
                    let summary_result = match prompt {
                        Some(prompt) => ai::Summary::from_audio(audio, Some(prompt), local),
                        None => ai::Summary::from_audio(audio, None, local),
                    }.await;


                    match summary_result {
                        Ok(summary) => {
                            if *print {
                                summary.print();
                            }

                            if *export {
                                if let Ok(_) = file_handling::export_summary(&summary) {
                                    println!(
                                        "\nSummary exported to {}/summaries",
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
            } else if let Some(interview) = interview {
                if !*print && !*export {
                    println!("No flag given for summary to print or export. Use `--print` and/or `--export` to print or export the summary.");
                } else {
                    let summary_result = ai::Summary::from_interview(interview).await;

                    match summary_result {
                        Ok(summary) => {
                            if *print {
                                summary.print();
                            }

                            if *export {
                                if let Ok(_) = file_handling::export_summary(&summary) {
                                    println!(
                                        "Summary exported to {}/summaries",
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
                    let summary_result = ai::Summary::from_prompt(prompt).await;

                    match summary_result {
                        Ok(summary) => {
                            if *print {
                                summary.print();
                            }

                            if *export {
                                if let Ok(_) = file_handling::export_summary(&summary) {
                                    println!(
                                        "Summary exported to {}/summaries",
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
            } else {
                println!("No valid import option provided. Use `--file`, `--note`, `--text` or `--list`.");
                Ok(())
            }


        }

        Some(Commands::Transcribe { audio, print, export, local ,}) => {
            if let Some(audio) = audio {
                if !*print && !*export {
                    println!("No flag given for summary to print or export. Use `--print` and/or `--export` to print or export the summary.");
                } else {
                    let transcription_result = if *local {
                        ai::Transcript::from_audio_local(audio).await
                    } else {
                        ai::Transcript::from_audio(audio).await
                    };

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
                                        "Unable to export transcript. Please check your permissions."
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
                println!("No valid import option provided. Use `--audio`.");
                Ok(())
            }
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
