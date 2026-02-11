//! Shell detection.

use std::env;

/// Detects the default shell for the current platform.
pub fn detect_shell() -> String {
    #[cfg(unix)]
    {
        env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    }
    #[cfg(windows)]
    {
        // Check for PowerShell Core
        if let Ok(path) = env::var("PATH") {
            for dir in env::split_paths(&path) {
                if dir.join("pwsh.exe").exists() {
                    return "pwsh.exe".to_string();
                }
            }
        }

        // Fallback to PowerShell
        "powershell.exe".to_string()
    }
    #[cfg(not(any(unix, windows)))]
    {
        "sh".to_string()
    }
}
