use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StoredCredentials {
    pub account: String,
    pub api_key: String,
}

#[derive(Clone, Debug)]
pub struct AccountSummary {
    pub name: String,
    pub account: String,
    pub current: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub current: Option<String>,
    pub accounts: BTreeMap<String, StoredCredentials>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ConfigFile {
    Multi(Config),
    Legacy(StoredCredentials),
}

impl Config {
    fn empty() -> Self {
        Self {
            current: None,
            accounts: BTreeMap::new(),
        }
    }
}

pub fn save_credentials(
    name: &str,
    account: &str,
    api_key: &str,
    make_current: bool,
) -> Result<PathBuf> {
    validate_name(name)?;

    if account.trim().is_empty() {
        bail!("account cannot be empty");
    }

    if api_key.trim().is_empty() {
        bail!("API key cannot be empty");
    }

    let mut config = load_config()?.unwrap_or_else(Config::empty);
    config.accounts.insert(
        name.to_owned(),
        StoredCredentials {
            account: account.to_owned(),
            api_key: api_key.to_owned(),
        },
    );

    if make_current || config.current.is_none() {
        config.current = Some(name.to_owned());
    }

    save_config(&config)
}

pub fn load_credentials(name: Option<&str>) -> Result<Option<StoredCredentials>> {
    let Some(config) = load_config()? else {
        return Ok(None);
    };

    let name = match name {
        Some(name) => name,
        None => match config.current.as_deref() {
            Some(current) => current,
            None => return Ok(None),
        },
    };

    Ok(config.accounts.get(name).cloned())
}

pub fn list_accounts() -> Result<Vec<AccountSummary>> {
    let Some(config) = load_config()? else {
        return Ok(Vec::new());
    };

    let current = config.current.as_deref();
    Ok(config
        .accounts
        .into_iter()
        .map(|(name, credentials)| AccountSummary {
            current: current == Some(name.as_str()),
            name,
            account: credentials.account,
        })
        .collect())
}

pub fn set_current_account(name: &str) -> Result<()> {
    validate_name(name)?;

    let mut config =
        load_config()?.context("no Simply accounts saved; run `simply login` first")?;
    if !config.accounts.contains_key(name) {
        bail!("no saved Simply account named `{name}`");
    }

    config.current = Some(name.to_owned());
    save_config(&config)?;
    Ok(())
}

pub fn remove_account(name: Option<&str>) -> Result<Option<String>> {
    let Some(mut config) = load_config()? else {
        return Ok(None);
    };

    let name = match name {
        Some(name) => name.to_owned(),
        None => match config.current.clone() {
            Some(current) => current,
            None => return Ok(None),
        },
    };

    let removed = config.accounts.remove(&name).is_some();
    if !removed {
        return Ok(None);
    }

    if config.current.as_deref() == Some(name.as_str()) {
        config.current = config.accounts.keys().next().cloned();
    }

    if config.accounts.is_empty() {
        delete_credentials_file()?;
    } else {
        save_config(&config)?;
    }

    Ok(Some(name))
}

pub fn delete_credentials_file() -> Result<()> {
    let path = config_path()?;
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| format!("failed to delete {}", path.display())),
    }
}

pub fn config_path() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("SIMPLY_CONFIG") {
        return Ok(PathBuf::from(path));
    }

    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(config_home)
            .join("simply")
            .join("config.json"));
    }

    let home = std::env::var_os("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("simply")
        .join("config.json"))
}

fn load_config() -> Result<Option<Config>> {
    let path = config_path()?;
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read {}", path.display()));
        }
    };

    let config_file: ConfigFile = serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    let config = match config_file {
        ConfigFile::Multi(config) => config,
        ConfigFile::Legacy(credentials) => {
            let mut accounts = BTreeMap::new();
            accounts.insert("default".to_owned(), credentials);
            Config {
                current: Some("default".to_owned()),
                accounts,
            }
        }
    };

    Ok(Some(config))
}

fn save_config(config: &Config) -> Result<PathBuf> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create Simply config directory {}",
                parent.display()
            )
        })?;
    }

    let contents =
        serde_json::to_vec_pretty(config).context("failed to serialize Simply config")?;
    write_private_file(&path, &contents)?;
    Ok(path)
}

fn validate_name(name: &str) -> Result<()> {
    if name.trim().is_empty() {
        bail!("account name cannot be empty");
    }

    Ok(())
}

#[cfg(unix)]
fn write_private_file(path: &PathBuf, contents: &[u8]) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .with_context(|| format!("failed to write {}", path.display()))?;
    file.write_all(contents)
        .with_context(|| format!("failed to write {}", path.display()))?;
    file.sync_all()
        .with_context(|| format!("failed to sync {}", path.display()))?;
    Ok(())
}

#[cfg(not(unix))]
fn write_private_file(path: &PathBuf, contents: &[u8]) -> Result<()> {
    fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))
}
