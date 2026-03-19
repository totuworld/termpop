use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
    #[serde(default = "default_font_size")]
    pub font_size: f64,
    #[serde(default = "default_width")]
    pub window_width: f64,
    #[serde(default = "default_height")]
    pub window_height: f64,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_strip_paste_hotkey")]
    pub strip_paste_hotkey: String,
}

fn default_hotkey() -> String {
    "Cmd+Shift+I".to_string()
}
fn default_font_size() -> f64 {
    14.0
}
fn default_width() -> f64 {
    600.0
}
fn default_height() -> f64 {
    300.0
}
fn default_theme() -> String {
    "dark".to_string()
}
fn default_strip_paste_hotkey() -> String {
    "Cmd+Shift+V".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: default_hotkey(),
            font_size: default_font_size(),
            window_width: default_width(),
            window_height: default_height(),
            theme: default_theme(),
            strip_paste_hotkey: default_strip_paste_hotkey(),
        }
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("termpop")
        .join("config.toml")
}

pub fn load_config() -> Config {
    let path = config_path();
    load_config_from(&path)
}

pub fn load_config_from(path: &std::path::Path) -> Config {
    match std::fs::read_to_string(path) {
        Ok(content) => parse_config(&content),
        Err(_) => Config::default(),
    }
}

pub fn save_config(config: &Config) -> Result<(), std::io::Error> {
    let path = config_path();
    save_config_to(config, &path)
}

pub fn save_config_to(config: &Config, path: &std::path::Path) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, content)
}

pub fn parse_config(content: &str) -> Config {
    toml::from_str(content).unwrap_or_default()
}

pub fn parse_hotkey(
    hotkey_str: &str,
) -> Option<(
    global_hotkey::hotkey::Modifiers,
    global_hotkey::hotkey::Code,
)> {
    use global_hotkey::hotkey::{Code, Modifiers};

    let parts: Vec<&str> = hotkey_str.split('+').map(|s| s.trim()).collect();
    if parts.len() < 2 {
        return None;
    }

    let mut modifiers = Modifiers::empty();
    for part in &parts[..parts.len() - 1] {
        match part.to_lowercase().as_str() {
            "cmd" | "super" | "command" => modifiers |= Modifiers::SUPER,
            "shift" => modifiers |= Modifiers::SHIFT,
            "alt" | "option" => modifiers |= Modifiers::ALT,
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            _ => return None,
        }
    }

    let key = match parts.last()?.to_uppercase().as_str() {
        "A" => Code::KeyA,
        "B" => Code::KeyB,
        "C" => Code::KeyC,
        "D" => Code::KeyD,
        "E" => Code::KeyE,
        "F" => Code::KeyF,
        "G" => Code::KeyG,
        "H" => Code::KeyH,
        "I" => Code::KeyI,
        "J" => Code::KeyJ,
        "K" => Code::KeyK,
        "L" => Code::KeyL,
        "M" => Code::KeyM,
        "N" => Code::KeyN,
        "O" => Code::KeyO,
        "P" => Code::KeyP,
        "Q" => Code::KeyQ,
        "R" => Code::KeyR,
        "S" => Code::KeyS,
        "T" => Code::KeyT,
        "U" => Code::KeyU,
        "V" => Code::KeyV,
        "W" => Code::KeyW,
        "X" => Code::KeyX,
        "Y" => Code::KeyY,
        "Z" => Code::KeyZ,
        _ => return None,
    };

    Some((modifiers, key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = Config::default();
        assert_eq!(config.hotkey, "Cmd+Shift+I");
        assert_eq!(config.font_size, 14.0);
        assert_eq!(config.window_width, 600.0);
        assert_eq!(config.window_height, 300.0);
        assert_eq!(config.theme, "dark");
        assert_eq!(config.strip_paste_hotkey, "Cmd+Shift+V");
    }

    #[test]
    fn parse_empty_string_returns_default() {
        let config = parse_config("");
        assert_eq!(config, Config::default());
    }

    #[test]
    fn parse_partial_config_fills_defaults() {
        let toml = r#"
hotkey = "Cmd+Shift+E"
font_size = 18.0
"#;
        let config = parse_config(toml);
        assert_eq!(config.hotkey, "Cmd+Shift+E");
        assert_eq!(config.font_size, 18.0);
        assert_eq!(config.window_width, 600.0);
        assert_eq!(config.window_height, 300.0);
    }

    #[test]
    fn parse_full_config() {
        let toml = r#"
hotkey = "Cmd+Shift+T"
font_size = 20.0
window_width = 800.0
window_height = 400.0
"#;
        let config = parse_config(toml);
        assert_eq!(config.hotkey, "Cmd+Shift+T");
        assert_eq!(config.font_size, 20.0);
        assert_eq!(config.window_width, 800.0);
        assert_eq!(config.window_height, 400.0);
    }

    #[test]
    fn parse_invalid_toml_returns_default() {
        let config = parse_config("this is not valid toml {{{}}}");
        assert_eq!(config, Config::default());
    }

    #[test]
    fn load_config_from_missing_file_returns_default() {
        let path = std::path::Path::new("/tmp/termpop_nonexistent_config.toml");
        let config = load_config_from(path);
        assert_eq!(config, Config::default());
    }

    #[test]
    fn load_config_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "font_size = 24.0\n").unwrap();
        let config = load_config_from(&path);
        assert_eq!(config.font_size, 24.0);
        assert_eq!(config.hotkey, "Cmd+Shift+I");
    }

    #[test]
    fn config_path_ends_with_expected() {
        let path = config_path();
        assert!(path.ends_with("termpop/config.toml"));
    }

    #[test]
    fn config_serializes_to_toml() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("hotkey"));
        assert!(toml_str.contains("font_size"));
    }

    #[test]
    fn parse_hotkey_cmd_shift_i() {
        use global_hotkey::hotkey::{Code, Modifiers};
        let result = parse_hotkey("Cmd+Shift+I");
        assert_eq!(
            result,
            Some((Modifiers::SUPER | Modifiers::SHIFT, Code::KeyI))
        );
    }

    #[test]
    fn parse_hotkey_cmd_shift_e() {
        use global_hotkey::hotkey::{Code, Modifiers};
        let result = parse_hotkey("Cmd+Shift+E");
        assert_eq!(
            result,
            Some((Modifiers::SUPER | Modifiers::SHIFT, Code::KeyE))
        );
    }

    #[test]
    fn parse_hotkey_ctrl_alt_t() {
        use global_hotkey::hotkey::{Code, Modifiers};
        let result = parse_hotkey("Ctrl+Alt+T");
        assert_eq!(
            result,
            Some((Modifiers::CONTROL | Modifiers::ALT, Code::KeyT))
        );
    }

    #[test]
    fn parse_hotkey_single_key_returns_none() {
        assert_eq!(parse_hotkey("A"), None);
    }

    #[test]
    fn parse_hotkey_invalid_modifier_returns_none() {
        assert_eq!(parse_hotkey("Foo+A"), None);
    }

    #[test]
    fn parse_hotkey_invalid_key_returns_none() {
        assert_eq!(parse_hotkey("Cmd+1"), None);
    }

    #[test]
    fn save_and_reload_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("termpop").join("config.toml");
        let mut config = Config::default();
        config.font_size = 32.0;
        save_config_to(&config, &path).unwrap();
        let loaded = load_config_from(&path);
        assert_eq!(loaded.font_size, 32.0);
        assert_eq!(loaded.hotkey, config.hotkey);
    }

    #[test]
    fn save_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("deep").join("nested").join("config.toml");
        let config = Config::default();
        assert!(save_config_to(&config, &path).is_ok());
        assert!(path.exists());
    }

    #[test]
    fn parse_config_with_theme() {
        let toml = r#"theme = "light""#;
        let config = parse_config(toml);
        assert_eq!(config.theme, "light");
    }

    #[test]
    fn save_and_reload_theme() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut config = Config::default();
        config.theme = "light".to_string();
        save_config_to(&config, &path).unwrap();
        let loaded = load_config_from(&path);
        assert_eq!(loaded.theme, "light");
    }
}
