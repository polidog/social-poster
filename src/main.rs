use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::{env, fs, io::Read, path::PathBuf, process::ExitCode, thread};

mod service;
mod tui;

use service::Service;

/// Post to Misskey, Mastodon and Bluesky at once.
#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    /// Post only to these accounts (comma separated names from the config)
    #[arg(long, value_delimiter = ',')]
    to: Vec<String>,
    /// Text to post. Read from stdin when omitted
    text: Vec<String>,
}

#[derive(Subcommand)]
enum Command {
    /// Open the interactive composer
    Tui,
}

#[derive(Deserialize)]
struct Config {
    accounts: Vec<Account>,
}

#[derive(Deserialize)]
pub struct Account {
    pub name: String,
    #[serde(flatten)]
    service: Service,
}

#[derive(Debug, thiserror::Error)]
enum ConfigError {
    #[error("cannot read {0}: {1}")]
    Read(PathBuf, std::io::Error),
    #[error("invalid config {0}: {1}")]
    Parse(PathBuf, toml::de::Error),
}

pub fn post_all(accounts: &[&Account], text: &str) -> Vec<(String, Result<String, service::Error>)> {
    thread::scope(|sc| {
        let hs: Vec<_> = accounts
            .iter()
            .map(|a| sc.spawn(move || (a.name.clone(), a.service.post(text))))
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

fn config_path() -> PathBuf {
    if let Some(p) = env::var_os("SOCIAL_POSTER_CONFIG") {
        return p.into();
    }
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".config"));
    base.join("social-poster/config.toml")
}

fn load_config() -> Result<Config, ConfigError> {
    let path = config_path();
    let s = fs::read_to_string(&path).map_err(|e| ConfigError::Read(path.clone(), e))?;
    toml::from_str(&s).map_err(|e| ConfigError::Parse(path, e))
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let cfg = match load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    if let Some(Command::Tui) = cli.command {
        return match tui::run(&cfg.accounts) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        };
    }

    if let Some(unknown) = cli.to.iter().find(|n| !cfg.accounts.iter().any(|a| &a.name == *n)) {
        eprintln!("unknown account: {unknown}");
        return ExitCode::FAILURE;
    }
    let targets: Vec<&Account> = cfg
        .accounts
        .iter()
        .filter(|a| cli.to.is_empty() || cli.to.contains(&a.name))
        .collect();
    if targets.is_empty() {
        eprintln!("no accounts configured");
        return ExitCode::FAILURE;
    }

    let text = if cli.text.is_empty() {
        let mut s = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut s) {
            eprintln!("cannot read stdin: {e}");
            return ExitCode::FAILURE;
        }
        s.trim_end().to_string()
    } else {
        cli.text.join(" ")
    };
    if text.is_empty() {
        eprintln!("text is empty");
        return ExitCode::FAILURE;
    }

    let mut ok = true;
    for (name, r) in post_all(&targets, &text) {
        match r {
            Ok(url) => println!("✓ {name} {url}"),
            Err(e) => {
                ok = false;
                eprintln!("✗ {name} {e}");
            }
        }
    }
    if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
