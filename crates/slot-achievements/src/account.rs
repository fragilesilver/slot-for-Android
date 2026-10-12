//! The account the player sets up from the in-app menu: read and written without starting the
//! service, so the menu works whether or not achievements are running.
use std::path::{Path, PathBuf};

use crate::storage::Config;

const FILE: &str = "Config/retroachievements.toml";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Account {
    pub enabled: bool,
    pub username: String,
    pub has_password: bool,
    pub has_token: bool,
}

fn path(root: &Path) -> PathBuf {
    root.join(FILE)
}

pub fn read_account(root: &Path) -> Account {
    match Config::read(&path(root)) {
        Ok(c) => Account {
            enabled: c.enabled,
            username: c.username,
            has_password: !c.password.is_empty(),
            has_token: !c.token.is_empty(),
        },
        Err(_) => Account::default(),
    }
}

fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn write(root: &Path, enabled: bool, username: &str, password: &str, token: &str) {
    let mut text = String::from(
        "# Written by slot's Achievements menu. After a successful login slot keeps a token and\n\
         # the password is removed.\n",
    );
    text.push_str(&format!("enabled = {enabled}\n"));
    text.push_str(&format!("username = {}\n", quote(username)));
    if !password.is_empty() {
        text.push_str(&format!("password = {}\n", quote(password)));
    }
    if !token.is_empty() {
        text.push_str(&format!("token = {}\n", quote(token)));
    }
    let target = path(root);
    if let Some(dir) = target.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let tmp = target.with_extension("toml.tmp");
    if std::fs::write(&tmp, text).is_ok() {
        let _ = std::fs::rename(&tmp, &target);
    }
}

/// Turn achievements on or off, keeping whatever login is stored.
pub fn set_enabled(root: &Path, enabled: bool) {
    let c = Config::read(&path(root)).unwrap_or_default();
    write(root, enabled, &c.username, &c.password, &c.token);
}

/// Store a new login. A different name or a new password drops the old token, which belonged
/// to the previous login.
pub fn set_login(root: &Path, username: &str, password: &str) {
    let c = Config::read(&path(root)).unwrap_or_default();
    let same = c.username.eq_ignore_ascii_case(username);
    let token = if same && password.is_empty() { c.token } else { String::new() };
    let enabled = c.enabled || !username.trim().is_empty();
    write(root, enabled, username.trim(), password, &token);
}

/// Forget the login: password and token go, and so does the cached session for the account.
/// Earned achievements and downloaded sets stay.
pub fn log_out(root: &Path) {
    let c = Config::read(&path(root)).unwrap_or_default();
    write(root, false, &c.username, "", "");
    let account = format!("{:x}", md5::compute(c.username.to_lowercase()));
    let _ = std::fs::remove_file(
        root.join("Saves/RetroAchievements")
            .join(account)
            .join("auth.json"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_login_round_trips_including_awkward_characters() {
        let d = tempfile::tempdir().unwrap();
        set_login(d.path(), "Player_1", "pa\"ss\\w0rd#=");
        let a = read_account(d.path());
        assert!(a.enabled && a.has_password && !a.has_token);
        assert_eq!(a.username, "Player_1");
        let c = Config::read(&path(d.path())).unwrap();
        assert_eq!(c.password, "pa\"ss\\w0rd#=");
    }

    #[test]
    fn disabling_keeps_the_login_and_logging_out_clears_it() {
        let d = tempfile::tempdir().unwrap();
        set_login(d.path(), "someone", "secret");
        set_enabled(d.path(), false);
        let a = read_account(d.path());
        assert!(!a.enabled && a.has_password && a.username == "someone");
        log_out(d.path());
        let a = read_account(d.path());
        assert!(!a.enabled && !a.has_password && !a.has_token);
    }
}
