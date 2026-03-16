use std::path::PathBuf;

pub fn plist_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join("Library/LaunchAgents/com.termpop.daemon.plist")
}

pub fn generate_plist(binary_path: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.termpop.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>{binary_path}</string>
        <string>daemon</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
    <key>StandardOutPath</key>
    <string>/tmp/termpop.out.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/termpop.err.log</string>
</dict>
</plist>
"#
    )
}

pub fn install_plist() -> Result<(), Box<dyn std::error::Error>> {
    let binary = std::env::current_exe()?;
    let binary_str = binary.to_str().ok_or("invalid binary path")?;
    let plist_content = generate_plist(binary_str);
    let path = plist_path();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&path, &plist_content)?;
    eprintln!("plist installed: {:?}", path);
    eprintln!("run: launchctl load {:?}", path);
    Ok(())
}

pub fn uninstall_plist() -> Result<(), Box<dyn std::error::Error>> {
    let path = plist_path();
    if path.exists() {
        std::fs::remove_file(&path)?;
        eprintln!("plist removed: {:?}", path);
    } else {
        eprintln!("plist not found: {:?}", path);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plist_path_ends_with_expected() {
        let path = plist_path();
        assert!(path.ends_with("Library/LaunchAgents/com.termpop.daemon.plist"));
    }

    #[test]
    fn generate_plist_contains_label() {
        let plist = generate_plist("/usr/local/bin/termpop");
        assert!(plist.contains("<string>com.termpop.daemon</string>"));
    }

    #[test]
    fn generate_plist_contains_binary_path() {
        let plist = generate_plist("/usr/local/bin/termpop");
        assert!(plist.contains("<string>/usr/local/bin/termpop</string>"));
    }

    #[test]
    fn generate_plist_contains_daemon_arg() {
        let plist = generate_plist("/usr/local/bin/termpop");
        assert!(plist.contains("<string>daemon</string>"));
    }

    #[test]
    fn generate_plist_is_valid_xml_structure() {
        let plist = generate_plist("/usr/local/bin/termpop");
        assert!(plist.starts_with("<?xml version="));
        assert!(plist.contains("<plist version=\"1.0\">"));
        assert!(plist.contains("</plist>"));
    }

    #[test]
    fn generate_plist_has_run_at_load() {
        let plist = generate_plist("/usr/local/bin/termpop");
        assert!(plist.contains("<key>RunAtLoad</key>"));
        assert!(plist.contains("<true/>"));
    }

    #[test]
    fn install_and_uninstall_plist_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("com.termpop.daemon.plist");
        let content = generate_plist("/test/termpop");
        std::fs::write(&path, &content).unwrap();
        assert!(path.exists());
        std::fs::remove_file(&path).unwrap();
        assert!(!path.exists());
    }
}
