use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use directories::ProjectDirs;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QuickCommand {
    pub name: String,
    pub command: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerConnection {
    pub nickname: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub ssh_key: Option<String>,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub quick_commands: Option<Vec<QuickCommand>>,
}

fn get_config_path() -> Result<PathBuf, String> {
    let proj_dirs = ProjectDirs::from("com", "termos", "termos")
        .ok_or_else(|| "Could not determine project directories".to_string())?;
    let config_dir = proj_dirs.config_dir().to_path_buf();
    
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }
    
    Ok(config_dir.join("connections.json"))
}

pub fn load_connections() -> Result<Vec<ServerConnection>, String> {
    let path = get_config_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let mut file = File::open(&path)
        .map_err(|e| format!("Failed to open connections file: {}", e))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| format!("Failed to read connections file: {}", e))?;

    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    let connections: Vec<ServerConnection> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse connections JSON: {}", e))?;
    Ok(connections)
}

pub fn save_connections(connections: &[ServerConnection]) -> Result<(), String> {
    let path = get_config_path()?;
    
    let json_content = serde_json::to_string_pretty(connections)
        .map_err(|e| format!("Failed to serialize connections: {}", e))?;

    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    let mut file = options.open(&path)
        .map_err(|e| format!("Failed to open connections file for writing: {}", e))?;

    file.write_all(json_content.as_bytes())
        .map_err(|e| format!("Failed to write connections to file: {}", e))?;

    Ok(())
}

pub fn add_connection(conn: ServerConnection) -> Result<(), String> {
    let mut conns = load_connections()?;
    if conns.iter().any(|c| c.nickname.eq_ignore_ascii_case(&conn.nickname)) {
        return Err(format!("A connection with nickname '{}' already exists.", conn.nickname));
    }
    conns.push(conn);
    save_connections(&conns)?;
    Ok(())
}

pub fn get_connection(nickname: &str) -> Result<Option<ServerConnection>, String> {
    let conns = load_connections()?;
    let found = conns.into_iter().find(|c| c.nickname.eq_ignore_ascii_case(nickname));
    Ok(found)
}

pub fn delete_connection(nickname: &str) -> Result<bool, String> {
    let mut conns = load_connections()?;
    let original_len = conns.len();
    conns.retain(|c| !c.nickname.eq_ignore_ascii_case(nickname));
    if conns.len() != original_len {
        save_connections(&conns)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn update_connection(old_nickname: &str, updated: ServerConnection) -> Result<bool, String> {
    let mut conns = load_connections()?;
    let mut found = false;

    if !old_nickname.eq_ignore_ascii_case(&updated.nickname) {
        if conns.iter().any(|c| c.nickname.eq_ignore_ascii_case(&updated.nickname)) {
            return Err(format!("Server with nickname '{}' already exists.", updated.nickname));
        }
    }

    for conn in conns.iter_mut() {
        if conn.nickname.eq_ignore_ascii_case(old_nickname) {
            *conn = updated.clone();
            found = true;
            break;
        }
    }

    if found {
        save_connections(&conns)?;
    }
    Ok(found)
}

pub fn add_quick_command(nickname: &str, qc: QuickCommand) -> Result<(), String> {
    let mut conns = load_connections()?;
    let mut found = false;
    for conn in conns.iter_mut() {
        if conn.nickname.eq_ignore_ascii_case(nickname) {
            let mut list = conn.quick_commands.take().unwrap_or_default();
            if list.iter().any(|c| c.name.eq_ignore_ascii_case(&qc.name)) {
                return Err(format!("Quick command '{}' already exists on this server.", qc.name));
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
    let mut conns = load_connections()?;
    let mut found_conn = false;
    let mut found_cmd = false;
    for conn in conns.iter_mut() {
        if conn.nickname.eq_ignore_ascii_case(nickname) {
            found_conn = true;
            if let Some(ref mut list) = conn.quick_commands {
                if let Some(ref n_name) = new_name {
                    if !n_name.eq_ignore_ascii_case(name) && list.iter().any(|c| c.name.eq_ignore_ascii_case(n_name)) {
                        return Err(format!("Quick command '{}' already exists.", n_name));
                    }
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
        return Err(format!("Quick command '{}' not found on this server.", name));
    }
    save_connections(&conns)
}

pub fn delete_quick_command(nickname: &str, name: &str) -> Result<(), String> {
    let mut conns = load_connections()?;
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
        return Err(format!("Quick command '{}' not found on this server.", name));
    }
    save_connections(&conns)
}
