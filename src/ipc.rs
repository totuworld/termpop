use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Request {
    Open {
        initial_text: Option<String>,
        title: Option<String>,
    },
    Status,
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Response {
    Result { text: String, cancelled: bool },
    Status { running: bool, hotkey: String },
    Ok,
}

pub fn socket_path() -> std::path::PathBuf {
    let support = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    support.join("termpop").join("daemon.sock")
}

pub fn encode_message(data: &[u8]) -> Vec<u8> {
    let len = (data.len() as u32).to_be_bytes();
    let mut buf = Vec::with_capacity(4 + data.len());
    buf.extend_from_slice(&len);
    buf.extend_from_slice(data);
    buf
}

pub fn decode_length(header: &[u8; 4]) -> usize {
    u32::from_be_bytes(*header) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_open_serializes_with_tag() {
        let req = Request::Open {
            initial_text: Some("hello".into()),
            title: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains(r#""type":"Open"#));
        assert!(json.contains(r#""initial_text":"hello"#));
    }

    #[test]
    fn request_status_roundtrip() {
        let req = Request::Status;
        let json = serde_json::to_string(&req).unwrap();
        let decoded: Request = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, Request::Status);
    }

    #[test]
    fn request_shutdown_roundtrip() {
        let req = Request::Shutdown;
        let json = serde_json::to_string(&req).unwrap();
        let decoded: Request = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, Request::Shutdown);
    }

    #[test]
    fn response_result_roundtrip() {
        let resp = Response::Result {
            text: "some text".into(),
            cancelled: false,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let decoded: Response = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, resp);
    }

    #[test]
    fn response_cancelled_roundtrip() {
        let resp = Response::Result {
            text: String::new(),
            cancelled: true,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let decoded: Response = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, resp);
    }

    #[test]
    fn response_status_roundtrip() {
        let resp = Response::Status {
            running: true,
            hotkey: "Cmd+Shift+E".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let decoded: Response = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, resp);
    }

    #[test]
    fn response_ok_roundtrip() {
        let resp = Response::Ok;
        let json = serde_json::to_string(&resp).unwrap();
        let decoded: Response = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, resp);
    }

    #[test]
    fn encode_decode_length_roundtrip() {
        let payload = b"hello world";
        let encoded = encode_message(payload);
        assert_eq!(encoded.len(), 4 + payload.len());

        let mut header = [0u8; 4];
        header.copy_from_slice(&encoded[..4]);
        let len = decode_length(&header);
        assert_eq!(len, payload.len());
        assert_eq!(&encoded[4..], payload);
    }

    #[test]
    fn encode_empty_message() {
        let encoded = encode_message(b"");
        assert_eq!(encoded.len(), 4);
        let mut header = [0u8; 4];
        header.copy_from_slice(&encoded[..4]);
        assert_eq!(decode_length(&header), 0);
    }

    #[test]
    fn socket_path_ends_with_daemon_sock() {
        let path = socket_path();
        assert!(path.ends_with("termpop/daemon.sock"));
    }
}
