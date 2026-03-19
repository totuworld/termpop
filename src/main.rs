mod ax_position;
mod cli;
mod clipboard;
mod config;
mod daemon;
mod editor;
mod ipc;
mod launchd;

use clap::Parser;
use cli::{Cli, Command};
use editor::{EditorConfig, EditorResult};
use ipc::{Request, Response};

fn main() {
    let cli = Cli::parse();
    let mut cfg = config::load_config();

    if let Some(fs) = cli.font_size {
        cfg.font_size = fs;
    }

    match cli.command {
        None => {
            let initial = cli.initial.clone();
            let title = cli.title.clone();

            if daemon::daemon_is_running() {
                let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
                let fallback_initial = initial.clone();
                let fallback_title = title.clone();
                let cfg_clone = cfg.clone();
                rt.block_on(async {
                    match open_via_daemon(initial, title).await {
                        Ok(EditorResult::Submitted(text)) => print!("{}", text),
                        Ok(EditorResult::Cancelled) => std::process::exit(1),
                        Err(e) => {
                            eprintln!("daemon connection failed: {}, falling back to direct mode", e);
                            run_direct(fallback_initial, fallback_title, &cfg_clone);
                        }
                    }
                });
            } else {
                run_direct(initial, title, &cfg);
            }
        }
        Some(Command::Daemon { install, uninstall }) => {
            if install {
                if let Err(e) = launchd::install_plist() {
                    eprintln!("failed to install plist: {}", e);
                    std::process::exit(1);
                }
                return;
            }
            if uninstall {
                if let Err(e) = launchd::uninstall_plist() {
                    eprintln!("failed to uninstall plist: {}", e);
                    std::process::exit(1);
                }
                return;
            }
            run_daemon(&cfg);
        }
        Some(Command::StripPaste) => {
            if daemon::daemon_is_running() {
                let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
                rt.block_on(async {
                    match send_strip_paste().await {
                        Ok(_) => {}
                        Err(e) => eprintln!("failed to send strip-paste: {}", e),
                    }
                });
            } else {
                clipboard::strip_and_paste(None);
            }
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

fn run_daemon(cfg: &config::Config) {
    use global_hotkey::hotkey::HotKey;
    use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
    use objc2_app_kit::*;
    use objc2_foundation::*;

    let (modifiers, code) = config::parse_hotkey(&cfg.hotkey).unwrap_or_else(|| {
        eprintln!("invalid hotkey '{}', using default Cmd+Shift+I", cfg.hotkey);
        config::parse_hotkey("Cmd+Shift+I").unwrap()
    });

    let strip_paste_hotkey_parsed = config::parse_hotkey(&cfg.strip_paste_hotkey).unwrap_or_else(|| {
        eprintln!("invalid strip_paste_hotkey '{}', using default Cmd+Shift+V", cfg.strip_paste_hotkey);
        config::parse_hotkey("Cmd+Shift+V").unwrap()
    });

    unsafe {
        let mtm = MainThreadMarker::new().expect("must be called from main thread");
        let app = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

        let manager = GlobalHotKeyManager::new().expect("failed to create hotkey manager");
        let hotkey = HotKey::new(Some(modifiers), code);
        manager
            .register(hotkey)
            .unwrap_or_else(|e| panic!("failed to register hotkey '{}': {}", cfg.hotkey, e));
        eprintln!("global hotkey registered: {}", cfg.hotkey);

        let strip_hotkey = HotKey::new(Some(strip_paste_hotkey_parsed.0), strip_paste_hotkey_parsed.1);
        manager
            .register(strip_hotkey)
            .unwrap_or_else(|e| panic!("failed to register strip_paste_hotkey '{}': {}", cfg.strip_paste_hotkey, e));
        eprintln!("strip paste hotkey registered: {}", cfg.strip_paste_hotkey);

        let hotkey_receiver = GlobalHotKeyEvent::receiver();

        let (action_tx, mut action_rx) = tokio::sync::mpsc::channel::<daemon::DaemonAction>(1);
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
            rt.block_on(async {
                if let Err(e) = daemon::run_socket_server(action_tx, shutdown_tx).await {
                    eprintln!("daemon error: {}", e);
                }
            });
        });

        loop {
            let event = app.nextEventMatchingMask_untilDate_inMode_dequeue(
                NSEventMask::Any,
                Some(&NSDate::dateWithTimeIntervalSinceNow(0.05)),
                NSDefaultRunLoopMode,
                true,
            );

            if let Some(ref event) = event {
                app.sendEvent(event);
            }

            if let Ok(hk_event) = hotkey_receiver.try_recv() {
                if hk_event.id == hotkey.id() && hk_event.state == HotKeyState::Pressed {
                    let previous_app = clipboard::get_frontmost_app();

                    let fresh_cfg = config::load_config();
                    let config = EditorConfig {
                        font_size: fresh_cfg.font_size,
                        width: fresh_cfg.window_width,
                        height: fresh_cfg.window_height,
                        theme: fresh_cfg.theme,
                        ..Default::default()
                    };
                    let result = editor::run_editor(config);

                    if let EditorResult::Submitted(text) = result {
                        clipboard::paste_text_and_restore(&text, previous_app.as_deref());
                    }
                }

                if hk_event.id == strip_hotkey.id() && hk_event.state == HotKeyState::Pressed {
                    let previous_app = clipboard::get_frontmost_app();
                    clipboard::strip_and_paste(previous_app.as_deref());
                }
            }

            if let Ok(action) = action_rx.try_recv() {
                match action {
                    daemon::DaemonAction::OpenEditor(req) => {
                        let result = editor::run_editor(req.config);
                        let _ = req.respond.send(result);
                    }
                    daemon::DaemonAction::StripPaste => {
                        let previous_app = clipboard::get_frontmost_app();
                        clipboard::strip_and_paste(previous_app.as_deref());
                    }
                }
            }

            if shutdown_rx.try_recv().is_ok() {
                break;
            }
        }
    }
}

fn run_direct(initial: Option<String>, title: Option<String>, cfg: &config::Config) {
    let config = EditorConfig {
        initial_text: initial.unwrap_or_default(),
        title: title.unwrap_or_else(|| "TermPop".to_string()),
        font_size: cfg.font_size,
        width: cfg.window_width,
        height: cfg.window_height,
        theme: cfg.theme.clone(),
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

async fn send_strip_paste() -> Result<Response, std::io::Error> {
    let mut stream = daemon::connect_to_daemon().await?;
    daemon::send_message(&mut stream, &Request::StripPaste).await?;
    daemon::recv_message(&mut stream).await
}
