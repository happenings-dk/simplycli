use anyhow::{Context, Result, anyhow, bail};
use keyring::Entry;

use crate::config;

const SERVICE: &str = "simply";
const ACCOUNT_ITEM: &str = "default-account";

#[derive(Debug)]
pub struct Credentials {
    pub account: String,
    pub api_key: String,
}

pub fn save_credentials(
    name: &str,
    account: &str,
    api_key: &str,
    make_current: bool,
) -> Result<()> {
    if account.trim().is_empty() {
        bail!("account cannot be empty");
    }

    if api_key.trim().is_empty() {
        bail!("API key cannot be empty");
    }

    config::save_credentials(name, account, api_key, make_current)?;

    if let Ok(entry) = account_entry() {
        let _ = entry.set_password(account);
    }

    if let Ok(entry) = api_key_entry(account) {
        let _ = entry.set_password(api_key);
    }

    Ok(())
}

pub fn load_credentials(
    profile: Option<String>,
    account_override: Option<String>,
    api_key_override: Option<String>,
) -> Result<Credentials> {
    let stored = config::load_credentials(profile.as_deref())?;

    let account = match account_override {
        Some(account) => account,
        None => stored
            .map(|credentials| credentials.account)
            .or_else(|| {
                account_entry()
                    .ok()
                    .and_then(|entry| entry.get_password().ok())
            })
            .context("pass --account, set SIMPLY_ACCOUNT, or run `simply login`")?,
    };

    let api_key = match api_key_override {
        Some(api_key) => api_key,
        None => config::load_credentials(profile.as_deref())?
            .filter(|credentials| credentials.account == account)
            .map(|credentials| credentials.api_key)
            .or_else(|| {
                api_key_entry(&account)
                    .ok()
                    .and_then(|entry| entry.get_password().ok())
            })
            .context("pass --api-key, set SIMPLY_API_KEY, or run `simply login`")?,
    };

    Ok(Credentials { account, api_key })
}

pub fn delete_credentials(name: Option<&str>, all: bool) -> Result<Option<String>> {
    if !all {
        return config::remove_account(name);
    }

    config::delete_credentials_file()?;

    let account = account_entry()
        .ok()
        .and_then(|entry| entry.get_password().ok());
    if let Some(account) = account
        && let Ok(entry) = api_key_entry(&account)
    {
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) | Err(keyring::Error::PlatformFailure(_)) => {}
            Err(_) => {}
        }
    }

    if let Ok(entry) = account_entry() {
        let _ = entry.delete_credential();
    }

    Ok(None)
}

fn account_entry() -> Result<Entry> {
    Entry::new(SERVICE, ACCOUNT_ITEM)
        .map_err(|error| anyhow!(error))
        .context("failed to open credential store")
}

fn api_key_entry(account: &str) -> Result<Entry> {
    Entry::new(SERVICE, account)
        .map_err(|error| anyhow!(error))
        .context("failed to open credential store")
}
