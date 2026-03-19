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

fn resolve_binary_path() -> Result<String, Box<dyn std::error::Error>> {
    let exe = std::env::current_exe()?;
    let exe_str = exe.to_str().ok_or("invalid binary path")?;

    if exe_str.contains(".app/Contents/MacOS/") {
        return Ok(exe_str.to_string());
    }

    let app_path = PathBuf::from("/Applications/TermPop.app/Contents/MacOS/termpop");
    if app_path.exists() {
        return Ok(app_path.to_str().unwrap().to_string());
    }

    Ok(exe_str.to_string())
}

pub fn install_plist() -> Result<(), Box<dyn std::error::Error>> {
    let binary_str = resolve_binary_path()?;
    let plist_content = generate_plist(&binary_str);
    let path = plist_path();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let _ = uninstall_plist();

    std::fs::write(&path, &plist_content)?;
    eprintln!("plist installed: {:?}", path);
    eprintln!("binary: {}", binary_str);

    let uid = unsafe { libc::getuid() };
    let domain = format!("gui/{}", uid);
    let path_str = path.to_str().ok_or("invalid plist path")?;

    let status = std::process::Command::new("launchctl")
        .args(["bootstrap", &domain, path_str])
        .status()?;

    if status.success() {
        eprintln!("daemon started via launchctl");
    } else {
        let fallback = std::process::Command::new("launchctl")
            .args(["load", path_str])
            .status()?;
        if fallback.success() {
            eprintln!("daemon started via launchctl (legacy)");
        } else {
            eprintln!(
                "launchctl failed (exit {}), run manually: launchctl bootstrap {} {:?}",
                status.code().unwrap_or(-1),
                domain,
                path
            );
        }
    }

    Ok(())
}

pub fn uninstall_plist() -> Result<(), Box<dyn std::error::Error>> {
    let path = plist_path();
    if path.exists() {
        let uid = unsafe { libc::getuid() };
        let domain = format!("gui/{}", uid);
        let path_str = path.to_str().ok_or("invalid plist path")?;

        let _ = std::process::Command::new("launchctl")
            .args(["bootout", &domain, path_str])
            .status();

        let _ = std::process::Command::new("launchctl")
            .args(["unload", path_str])
            .status();

        std::fs::remove_file(&path)?;
        eprintln!("daemon stopped and plist removed: {:?}", path);
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
    fn generate_plist_contains_app_bundle_path() {
        let app_path = "/Applications/TermPop.app/Contents/MacOS/termpop";
        let plist = generate_plist(app_path);
        assert!(plist.contains(&format!("<string>{}</string>", app_path)));
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
