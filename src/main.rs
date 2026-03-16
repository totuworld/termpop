mod cli;
mod editor;

use clap::Parser;
use cli::{Cli, Command};
use editor::{EditorConfig, EditorResult};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => {
            let config = EditorConfig {
                initial_text: cli.initial.unwrap_or_default(),
                title: cli.title.unwrap_or_else(|| "TermPop".to_string()),
                ..Default::default()
            };

            match editor::run_editor(config) {
                EditorResult::Submitted(text) => {
                    print!("{}", text);
                }
                EditorResult::Cancelled => {
                    std::process::exit(1);
                }
            }
        }
        Some(Command::Daemon { install: _ }) => {
            eprintln!("daemon mode not yet implemented");
            std::process::exit(2);
        }
        Some(Command::Status) => {
            eprintln!("status not yet implemented");
            std::process::exit(2);
        }
        Some(Command::Stop) => {
            eprintln!("stop not yet implemented");
            std::process::exit(2);
        }
    }
}
