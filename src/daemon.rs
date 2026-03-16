use crate::editor::{EditorConfig, EditorResult};
use crate::ipc::{self, Request, Response};
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;

pub struct EditorRequest {
    pub config: EditorConfig,
    pub respond: tokio::sync::oneshot::Sender<EditorResult>,
}

pub async fn send_message<T: serde::Serialize>(
    stream: &mut UnixStream,
    msg: &T,
) -> std::io::Result<()> {
    let json = serde_json::to_vec(msg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let encoded = ipc::encode_message(&json);
    stream.write_all(&encoded).await
}

pub async fn recv_message<T: serde::de::DeserializeOwned>(
    stream: &mut UnixStream,
) -> std::io::Result<T> {
    let mut header = [0u8; 4];
    stream.read_exact(&mut header).await?;
    let len = ipc::decode_length(&header);
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    serde_json::from_slice(&buf)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

pub fn ensure_socket_dir() -> std::io::Result<std::path::PathBuf> {
    let path = ipc::socket_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(path)
}

pub fn cleanup_socket(path: &Path) {
    let _ = std::fs::remove_file(path);
}

pub fn daemon_is_running() -> bool {
    let path = ipc::socket_path();
    path.exists()
}

pub async fn connect_to_daemon() -> std::io::Result<UnixStream> {
    let path = ipc::socket_path();
    UnixStream::connect(&path).await
}

pub async fn run_socket_server(
    editor_tx: mpsc::Sender<EditorRequest>,
    shutdown_tx: mpsc::Sender<()>,
) -> std::io::Result<()> {
    let sock_path = ensure_socket_dir()?;
    cleanup_socket(&sock_path);

    let listener = UnixListener::bind(&sock_path)?;
    eprintln!("daemon listening on {:?}", sock_path);

    loop {
        let (mut stream, _) = listener.accept().await?;
        let req: Request = match recv_message(&mut stream).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("failed to read request: {}", e);
                continue;
            }
        };

        match req {
            Request::Open {
                initial_text,
                title,
            } => {
                let config = EditorConfig {
                    initial_text: initial_text.unwrap_or_default(),
                    title: title.unwrap_or_else(|| "TermPop".to_string()),
                    ..Default::default()
                };

                let (tx, rx) = tokio::sync::oneshot::channel();
                if editor_tx.send(EditorRequest { config, respond: tx }).await.is_err() {
                    let resp = Response::Result {
                        text: String::new(),
                        cancelled: true,
                    };
                    let _ = send_message(&mut stream, &resp).await;
                    continue;
                }

                let result = rx.await.unwrap_or(EditorResult::Cancelled);
                let resp = match result {
                    EditorResult::Submitted(text) => Response::Result {
                        text,
                        cancelled: false,
                    },
                    EditorResult::Cancelled => Response::Result {
                        text: String::new(),
                        cancelled: true,
                    },
                };
                let _ = send_message(&mut stream, &resp).await;
            }
            Request::Status => {
                let resp = Response::Status {
                    running: true,
                    hotkey: "Cmd+Shift+E".into(),
                };
                let _ = send_message(&mut stream, &resp).await;
            }
            Request::Shutdown => {
                let _ = send_message(&mut stream, &Response::Ok).await;
                cleanup_socket(&sock_path);
                let _ = shutdown_tx.send(()).await;
                return Ok(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn send_recv_request_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("test.sock");

        let listener = UnixListener::bind(&sock_path).unwrap();

        let client_path = sock_path.clone();
        let client = tokio::spawn(async move {
            let mut stream = UnixStream::connect(&client_path).await.unwrap();
            let req = Request::Open {
                initial_text: Some("test".into()),
                title: None,
            };
            send_message(&mut stream, &req).await.unwrap();

            let resp: Response = recv_message(&mut stream).await.unwrap();
            resp
        });

        let (mut server_stream, _) = listener.accept().await.unwrap();
        let req: Request = recv_message(&mut server_stream).await.unwrap();
        assert_eq!(
            req,
            Request::Open {
                initial_text: Some("test".into()),
                title: None,
            }
        );

        let resp = Response::Result {
            text: "result text".into(),
            cancelled: false,
        };
        send_message(&mut server_stream, &resp).await.unwrap();

        let client_resp = client.await.unwrap();
        assert_eq!(
            client_resp,
            Response::Result {
                text: "result text".into(),
                cancelled: false,
            }
        );
    }

    #[tokio::test]
    async fn send_recv_shutdown_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("test.sock");

        let listener = UnixListener::bind(&sock_path).unwrap();

        let client_path = sock_path.clone();
        let client = tokio::spawn(async move {
            let mut stream = UnixStream::connect(&client_path).await.unwrap();
            send_message(&mut stream, &Request::Shutdown).await.unwrap();
            let resp: Response = recv_message(&mut stream).await.unwrap();
            resp
        });

        let (mut server_stream, _) = listener.accept().await.unwrap();
        let req: Request = recv_message(&mut server_stream).await.unwrap();
        assert_eq!(req, Request::Shutdown);

        send_message(&mut server_stream, &Response::Ok).await.unwrap();

        let client_resp = client.await.unwrap();
        assert_eq!(client_resp, Response::Ok);
    }

    #[test]
    fn ensure_socket_dir_creates_parent() {
        let path = ensure_socket_dir().unwrap();
        assert!(path.parent().unwrap().exists());
    }
}
