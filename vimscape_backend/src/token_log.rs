use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use crate::token::Token;

struct TokenLogConfig {
    enabled: bool,
    log_path: PathBuf,
}

static TOKEN_LOG_CONFIG: LazyLock<Mutex<TokenLogConfig>> = LazyLock::new(|| {
    Mutex::new(TokenLogConfig {
        enabled: false,
        log_path: PathBuf::new(),
    })
});

pub fn enable(db_path: &str) {
    let mut path = PathBuf::from(db_path);
    if path.extension().is_some() {
        path.pop();
    }
    path.push("vimscape_token_log.txt");

    if let Ok(mut file) = File::create(&path) {
        let _ = writeln!(file, "# Vimscape2007 Token Log");
        let _ = writeln!(file, "# Format: Token: <Debug representation>");
        let _ = writeln!(file, "# ========================================");
    }

    if let Ok(mut config) = TOKEN_LOG_CONFIG.lock() {
        config.enabled = true;
        config.log_path = path;
    }
}

pub fn is_enabled() -> bool {
    TOKEN_LOG_CONFIG
        .lock()
        .map(|config| config.enabled)
        .unwrap_or(false)
}

pub fn log_token(token: &Token) {
    let log_path = {
        let Ok(config) = TOKEN_LOG_CONFIG.lock() else {
            return;
        };
        if !config.enabled {
            return;
        }
        config.log_path.clone()
    };

    let Ok(mut file) = OpenOptions::new().append(true).open(&log_path) else {
        return;
    };

    let _ = writeln!(file, "Token: {token:?}");
}

pub fn log_batch(input: &str) {
    let log_path = {
        let Ok(config) = TOKEN_LOG_CONFIG.lock() else {
            return;
        };
        if !config.enabled {
            return;
        }
        config.log_path.clone()
    };

    let Ok(mut file) = OpenOptions::new().append(true).open(&log_path) else {
        return;
    };

    let _ = writeln!(file, "Batch: {input}");
}
