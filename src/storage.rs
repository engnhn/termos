use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use zeroize::Zeroize;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QuickCommand {
    pub name: String,
    pub command: String,
}

pub(crate) fn validate_quick_command_fields(name: &str, command: &str) -> Result<(), String> {
    if name.trim() != name
        || name.is_empty()
        || name.len() > 64
        || name.chars().any(char::is_control)
    {
        return Err(
            "Quick command name must be 1–64 bytes with no surrounding whitespace or control characters"
                .to_string(),
        );
    }
    if command.trim().is_empty() || command.len() > 4096 || command.contains('\0') {
        return Err(
            "Command must be non-empty, at most 4096 bytes, and contain no NUL characters"
                .to_string(),
        );
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerConnection {
    pub nickname: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password_ref: Option<String>,
    pub ssh_key: Option<String>,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub quick_commands: Option<Vec<QuickCommand>>,
}

impl Drop for ServerConnection {
    fn drop(&mut self) {
        if let Some(password) = &mut self.password {
            password.zeroize();
        }
    }
}

fn credential_entry(reference: &str) -> Result<keyring::Entry, String> {
    uuid::Uuid::parse_str(reference)
        .map_err(|_| "Invalid credential reference in connections file".to_string())?;
    keyring::Entry::new("termos", reference)
        .map_err(|e| format!("Could not access the system credential store: {}", e))
}

fn remove_credential(reference: &str) -> Result<(), String> {
    let entry = credential_entry(reference)?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!(
            "Could not remove a saved password from the system credential store: {}",
            e
        )),
    }
}

fn prepare_stored_connections<F>(
    connections: &[ServerConnection],
    mut set_password: F,
) -> Result<Vec<ServerConnection>, String>
where
    F: FnMut(&str, &str) -> Result<(), String>,
{
    let mut stored = connections.to_vec();
    for conn in &mut stored {
        if let Some(password) = conn.password.as_deref().filter(|p| !p.is_empty()) {
            if conn.password_ref.is_none() {
                conn.password_ref = Some(uuid::Uuid::new_v4().to_string());
            }
            set_password(conn.password_ref.as_deref().unwrap(), password)?;
        }
        conn.password = None;
    }
    Ok(stored)
}

fn get_config_path() -> Result<PathBuf, String> {
    let proj_dirs = ProjectDirs::from("com", "termos", "termos")
        .ok_or_else(|| "Could not determine project directories".to_string())?;
    let config_dir = proj_dirs.config_dir().to_path_buf();

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&config_dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| format!("Failed to secure config directory: {}", e))?;
    }

    Ok(config_dir.join("connections.json"))
}

fn lock_connections() -> Result<File, String> {
    let path = get_config_path()?.with_extension("lock");
    let mut options = OpenOptions::new();
    options.write(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options
        .open(path)
        .map_err(|e| format!("Failed to open connections lock: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } != 0 {
            return Err(format!(
                "Failed to lock connections file: {}",
                std::io::Error::last_os_error()
            ));
        }
    }
    Ok(file)
}

pub fn load_connections() -> Result<Vec<ServerConnection>, String> {
    let _lock = lock_connections()?;
    load_connections_unlocked()
}

fn load_connections_unlocked() -> Result<Vec<ServerConnection>, String> {
    let path = get_config_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let mut file =
        File::open(&path).map_err(|e| format!("Failed to open connections file: {}", e))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| format!("Failed to read connections file: {}", e))?;

    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut connections: Vec<ServerConnection> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse connections JSON: {}", e))?;

    let mut migrate_legacy_passwords = false;
    for conn in &mut connections {
        if conn.password.as_deref() == Some("") {
            conn.password = None;
        }
        if conn.password.is_some() && conn.password_ref.is_none() {
            conn.password_ref = Some(uuid::Uuid::new_v4().to_string());
            migrate_legacy_passwords = true;
        }
        if conn.password.is_none()
            && let Some(reference) = &conn.password_ref
        {
            conn.password = Some(credential_entry(reference)?.get_password().map_err(|e| {
                format!(
                    "Could not retrieve a saved password from the system credential store: {}",
                    e
                )
            })?);
        }
    }
    if migrate_legacy_passwords {
        save_connections(&connections)?;
    }
    Ok(connections)
}

fn save_connections(connections: &[ServerConnection]) -> Result<(), String> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

    let path = get_config_path()?;

    let stored_connections = prepare_stored_connections(connections, |reference, password| {
        credential_entry(reference)?
            .set_password(password)
            .map_err(|e| {
                format!(
                    "Could not save password to the system credential store: {}",
                    e
                )
            })
    })?;

    let json_content = serde_json::to_string_pretty(&stored_connections)
        .map_err(|e| format!("Failed to serialize connections: {}", e))?;

    let parent = path
        .parent()
        .ok_or_else(|| "Config path has no parent directory".to_string())?;
    let mut temp_path = parent.to_path_buf();
    let mut temp_file = None;
    for _ in 0..100 {
        temp_path.push(format!(
            "connections-{}-{}.tmp",
            std::process::id(),
            NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed)
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&temp_path) {
            Ok(file) => {
                temp_file = Some(file);
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                temp_path.pop();
            }
            Err(e) => {
                return Err(format!(
                    "Failed to create temporary connections file: {}",
                    e
                ));
            }
        }
    }
    let mut file = temp_file
        .ok_or_else(|| "Could not create a unique temporary connections file".to_string())?;
    let write_result = file
        .write_all(json_content.as_bytes())
        .and_then(|_| file.sync_all());
    drop(file);
    if let Err(e) = write_result {
        let _ = std::fs::remove_file(&temp_path);
        return Err(format!("Failed to write connections to file: {}", e));
    }
    if let Err(e) = std::fs::rename(&temp_path, &path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(format!("Failed to replace connections file: {}", e));
    }
    Ok(())
}

pub fn add_connection(conn: ServerConnection) -> Result<(), String> {
    let _lock = lock_connections()?;
    let mut conns = load_connections_unlocked()?;
    if conns
        .iter()
        .any(|c| c.nickname.eq_ignore_ascii_case(&conn.nickname))
    {
        return Err(format!(
            "A connection with nickname '{}' already exists.",
            conn.nickname
        ));
    }
    conns.push(conn);
    save_connections(&conns)?;
    Ok(())
}

pub fn get_connection(nickname: &str) -> Result<Option<ServerConnection>, String> {
    let conns = load_connections()?;
    let found = conns
        .into_iter()
        .find(|c| c.nickname.eq_ignore_ascii_case(nickname));
    Ok(found)
}

pub fn delete_connection(nickname: &str) -> Result<bool, String> {
    let _lock = lock_connections()?;
    let mut conns = load_connections_unlocked()?;
    let original_len = conns.len();
    let removed_refs: Vec<String> = conns
        .iter()
        .filter(|c| c.nickname.eq_ignore_ascii_case(nickname))
        .filter_map(|c| c.password_ref.clone())
        .collect();
    conns.retain(|c| !c.nickname.eq_ignore_ascii_case(nickname));
    if conns.len() != original_len {
        save_connections(&conns)?;
        for reference in removed_refs {
            remove_credential(&reference).map_err(|e| {
                format!(
                    "Connection was removed, but its saved credential cleanup failed: {}",
                    e
                )
            })?;
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn update_connection(old_nickname: &str, updated: ServerConnection) -> Result<bool, String> {
    let _lock = lock_connections()?;
    let mut conns = load_connections_unlocked()?;
    let mut found = false;
    let mut credential_to_remove = None;

    if !old_nickname.eq_ignore_ascii_case(&updated.nickname)
        && conns
            .iter()
            .any(|c| c.nickname.eq_ignore_ascii_case(&updated.nickname))
    {
        return Err(format!(
            "Server with nickname '{}' already exists.",
            updated.nickname
        ));
    }

    for conn in conns.iter_mut() {
        if conn.nickname.eq_ignore_ascii_case(old_nickname) {
            let old_password_ref = conn.password_ref.clone();
            *conn = updated.clone();
            if conn.password.is_none() {
                credential_to_remove = old_password_ref;
                conn.password_ref = None;
            } else if conn.password_ref.is_none() {
                conn.password_ref = old_password_ref;
            }
            found = true;
            break;
        }
    }

    if found {
        save_connections(&conns)?;
        if let Some(reference) = credential_to_remove {
            remove_credential(&reference).map_err(|e| {
                format!(
                    "Connection was updated, but its old credential cleanup failed: {}",
                    e
                )
            })?;
        }
    }
    Ok(found)
}

pub fn add_quick_command(nickname: &str, qc: QuickCommand) -> Result<(), String> {
    validate_quick_command_fields(&qc.name, &qc.command)?;
    let _lock = lock_connections()?;
    let mut conns = load_connections_unlocked()?;
    let mut found = false;
    for conn in conns.iter_mut() {
        if conn.nickname.eq_ignore_ascii_case(nickname) {
            let mut list = conn.quick_commands.take().unwrap_or_default();
            if list.iter().any(|c| c.name.eq_ignore_ascii_case(&qc.name)) {
                return Err(format!(
                    "Quick command '{}' already exists on this server.",
                    qc.name
                ));
            }
            list.push(qc.clone());
            conn.quick_commands = Some(list);
            found = true;
            break;
        }
    }
    if !found {
        return Err(format!("Server with nickname '{}' not found.", nickname));
    }
    save_connections(&conns)
}

pub fn edit_quick_command(
    nickname: &str,
    name: &str,
    new_name: Option<String>,
    new_cmd: Option<String>,
) -> Result<(), String> {
    if let Some(ref updated_name) = new_name
        && (updated_name.trim() != updated_name
            || updated_name.is_empty()
            || updated_name.len() > 64
            || updated_name.chars().any(char::is_control))
    {
        return Err(
            "Quick command name must be 1–64 bytes with no surrounding whitespace or control characters"
                .to_string(),
        );
    }
    if let Some(ref updated_command) = new_cmd
        && (updated_command.trim().is_empty()
            || updated_command.len() > 4096
            || updated_command.contains('\0'))
    {
        return Err(
            "Command must be non-empty, at most 4096 bytes, and contain no NUL characters"
                .to_string(),
        );
    }
    let _lock = lock_connections()?;
    let mut conns = load_connections_unlocked()?;
    let mut found_conn = false;
    let mut found_cmd = false;
    for conn in conns.iter_mut() {
        if conn.nickname.eq_ignore_ascii_case(nickname) {
            found_conn = true;
            if let Some(ref mut list) = conn.quick_commands {
                if let Some(ref n_name) = new_name
                    && !n_name.eq_ignore_ascii_case(name)
                    && list.iter().any(|c| c.name.eq_ignore_ascii_case(n_name))
                {
                    return Err(format!("Quick command '{}' already exists.", n_name));
                }
                for qc in list.iter_mut() {
                    if qc.name.eq_ignore_ascii_case(name) {
                        if let Some(ref n_name) = new_name {
                            qc.name = n_name.clone();
                        }
                        if let Some(ref n_cmd) = new_cmd {
                            qc.command = n_cmd.clone();
                        }
                        found_cmd = true;
                        break;
                    }
                }
            }
            break;
        }
    }
    if !found_conn {
        return Err(format!("Server with nickname '{}' not found.", nickname));
    }
    if !found_cmd {
        return Err(format!(
            "Quick command '{}' not found on this server.",
            name
        ));
    }
    save_connections(&conns)
}

pub fn delete_quick_command(nickname: &str, name: &str) -> Result<(), String> {
    let _lock = lock_connections()?;
    let mut conns = load_connections_unlocked()?;
    let mut found_conn = false;
    let mut found_cmd = false;
    for conn in conns.iter_mut() {
        if conn.nickname.eq_ignore_ascii_case(nickname) {
            found_conn = true;
            if let Some(ref mut list) = conn.quick_commands {
                let len_before = list.len();
                list.retain(|c| !c.name.eq_ignore_ascii_case(name));
                if list.len() < len_before {
                    found_cmd = true;
                }
                if list.is_empty() {
                    conn.quick_commands = None;
                }
            }
            break;
        }
    }
    if !found_conn {
        return Err(format!("Server with nickname '{}' not found.", nickname));
    }
    if !found_cmd {
        return Err(format!(
            "Quick command '{}' not found on this server.",
            name
        ));
    }
    save_connections(&conns)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connection(password: Option<&str>, password_ref: Option<&str>) -> ServerConnection {
        ServerConnection {
            nickname: "demo".to_string(),
            host: "example.org".to_string(),
            port: 22,
            username: "user".to_string(),
            password: password.map(str::to_string),
            password_ref: password_ref.map(str::to_string),
            ssh_key: None,
            group: None,
            quick_commands: None,
        }
    }

    #[test]
    fn saved_json_never_contains_password_value() {
        let input = vec![connection(Some("sensitive-value"), None)];
        let mut captured = None;
        let records = prepare_stored_connections(&input, |reference, password| {
            captured = Some((reference.to_string(), password.to_string()));
            Ok(())
        })
        .unwrap();
        let json = serde_json::to_string(&records).unwrap();
        assert!(!json.contains("sensitive-value"));
        assert!(!json.contains("\"password\":"));
        assert!(json.contains("password_ref"));
        assert_eq!(captured.unwrap().1, "sensitive-value");
        assert!(records[0].password.is_none());
        assert!(input[0].password.is_some());
    }

    #[test]
    fn credential_references_must_be_uuid_values() {
        assert!(credential_entry("../../connections.json").is_err());
    }

    #[test]
    fn quick_command_validation_is_shared_by_all_entry_points() {
        assert!(validate_quick_command_fields("Disk usage", "df -h").is_ok());
        assert!(validate_quick_command_fields("", "df -h").is_err());
        assert!(validate_quick_command_fields("Disk usage ", "df -h").is_err());
        assert!(validate_quick_command_fields("Disk usage", " \n ").is_err());
        assert!(validate_quick_command_fields("Disk usage", "x".repeat(4097).as_str()).is_err());
        assert!(validate_quick_command_fields("x".repeat(65).as_str(), "df -h").is_err());
        assert!(validate_quick_command_fields("Disk\nusage", "df -h").is_err());
        assert!(validate_quick_command_fields("Disk usage", "echo\0bad").is_err());
    }
}
