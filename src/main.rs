mod cli;
mod daemon;
mod editor;
mod ipc;

use clap::Parser;
use cli::{Cli, Command};
use editor::{EditorConfig, EditorResult};
use ipc::{Request, Response};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => {
            let initial = cli.initial.clone();
            let title = cli.title.clone();

            if daemon::daemon_is_running() {
                let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
                let fallback_initial = initial.clone();
                let fallback_title = title.clone();
                rt.block_on(async {
                    match open_via_daemon(initial, title).await {
                        Ok(EditorResult::Submitted(text)) => print!("{}", text),
                        Ok(EditorResult::Cancelled) => std::process::exit(1),
                        Err(e) => {
                            eprintln!("daemon connection failed: {}, falling back to direct mode", e);
                            run_direct(fallback_initial, fallback_title);
                        }
                    }
                });
            } else {
                run_direct(initial, title);
            }
        }
        Some(Command::Daemon { install: _ }) => {
            run_daemon();
        }
        Some(Command::Status) => {
            let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
            rt.block_on(async {
                match query_status().await {
                    Ok(Response::Status { running, hotkey }) => {
                        println!("running: {}, hotkey: {}", running, hotkey);
                    }
                    Ok(_) => eprintln!("unexpected response"),
                    Err(e) => eprintln!("daemon not running: {}", e),
                }
            });
        }
        Some(Command::Stop) => {
            let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
            rt.block_on(async {
                match send_shutdown().await {
                    Ok(_) => println!("daemon stopped"),
                    Err(e) => eprintln!("failed to stop daemon: {}", e),
                }
            });
        }
    }
}

fn run_daemon() {
    let (editor_tx, mut editor_rx) = tokio::sync::mpsc::channel::<daemon::EditorRequest>(1);
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(async {
            if let Err(e) = daemon::run_socket_server(editor_tx, shutdown_tx).await {
                eprintln!("daemon error: {}", e);
            }
        });
    });

    loop {
        if let Ok(req) = editor_rx.try_recv() {
            let result = editor::run_editor(req.config);
            let _ = req.respond.send(result);
        }

        if shutdown_rx.try_recv().is_ok() {
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

fn run_direct(initial: Option<String>, title: Option<String>) {
    let config = EditorConfig {
        initial_text: initial.unwrap_or_default(),
        title: title.unwrap_or_else(|| "TermPop".to_string()),
        ..Default::default()
    };

    match editor::run_editor(config) {
        EditorResult::Submitted(text) => print!("{}", text),
        EditorResult::Cancelled => std::process::exit(1),
    }
}

async fn open_via_daemon(
    initial: Option<String>,
    title: Option<String>,
) -> Result<EditorResult, std::io::Error> {
    let mut stream = daemon::connect_to_daemon().await?;
    let req = Request::Open {
        initial_text: initial,
        title,
    };
    daemon::send_message(&mut stream, &req).await?;
    let resp: Response = daemon::recv_message(&mut stream).await?;
    match resp {
        Response::Result { text, cancelled } => {
            if cancelled {
                Ok(EditorResult::Cancelled)
            } else {
                Ok(EditorResult::Submitted(text))
            }
        }
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "unexpected response from daemon",
        )),
    }
}

async fn query_status() -> Result<Response, std::io::Error> {
    let mut stream = daemon::connect_to_daemon().await?;
    daemon::send_message(&mut stream, &Request::Status).await?;
    daemon::recv_message(&mut stream).await
}

async fn send_shutdown() -> Result<Response, std::io::Error> {
    let mut stream = daemon::connect_to_daemon().await?;
    daemon::send_message(&mut stream, &Request::Shutdown).await?;
    daemon::recv_message(&mut stream).await
}
