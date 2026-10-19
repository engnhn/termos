use crate::storage::ServerConnection;
use std::io::{self, Write};
use std::process::Command;
use std::thread;
use std::time::Duration;

#[cfg(unix)]
fn configure_askpass(
    cmd: &mut Command,
    password: &str,
    current_exe: &std::path::Path,
) -> Result<Option<std::fs::File>, String> {
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::OpenOptionsExt;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);
    if password.chars().any(|c| matches!(c, '\n' | '\r' | '\0')) || password.len() > 65_536 {
        return Err("SSH password contains unsupported characters or exceeds 64 KiB".to_string());
    }
    let mut path = std::env::temp_dir();
    let mut secret_file = None;
    for _ in 0..100 {
        path.push(format!(
            "termos-secret-{}-{}",
            std::process::id(),
            NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed)
        ));
        match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
        {
            Ok(file) => {
                secret_file = Some(file);
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                path.pop();
            }
            Err(e) => return Err(format!("Failed to create temporary credential file: {}", e)),
        }
    }
    let mut file = secret_file
        .ok_or_else(|| "Could not create a unique temporary credential file".to_string())?;
    std::fs::remove_file(&path)
        .map_err(|e| format!("Failed to unlink temporary credential file: {}", e))?;
    file.write_all(password.as_bytes())
        .map_err(|e| format!("Failed to stage SSH credential: {}", e))?;

    let fd = file.as_raw_fd();
    // The file is intentionally inherited by ssh and its askpass callback; it has no filesystem name.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags == -1 || unsafe { libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) } == -1 {
        return Err(format!(
            "Failed to prepare SSH credential descriptor: {}",
            std::io::Error::last_os_error()
        ));
    }

    cmd.env("SSH_ASKPASS", current_exe)
        .env("SSH_ASKPASS_REQUIRE", "force")
        .env("TERMOS_ASKPASS_FD", fd.to_string())
        .env("TERMOS_ASKPASS_LEN", password.len().to_string());
    if std::env::var("DISPLAY").is_err() {
        cmd.env("DISPLAY", ":0");
    }
    Ok(Some(file))
}

#[cfg(not(unix))]
fn configure_askpass(
    cmd: &mut Command,
    password: &str,
    current_exe: &std::path::Path,
) -> Result<Option<std::fs::File>, String> {
    let _ = (cmd, password, current_exe);
    Err("Password-based SSH authentication is supported only on Unix; use an SSH key on this platform".to_string())
}

fn append_target_args(args: &mut Vec<String>, conn: &ServerConnection) -> Result<(), String> {
    let invalid =
        |value: &str| value.chars().any(char::is_whitespace) || value.chars().any(char::is_control);
    if conn.username.is_empty()
        || conn.username.starts_with('-')
        || conn.username.contains('@')
        || invalid(&conn.username)
    {
        return Err("Invalid SSH username; use a username without whitespace, control characters, '@', or a leading '-'".to_string());
    }
    if conn.host.is_empty()
        || conn.host.starts_with('-')
        || conn.host.contains('@')
        || invalid(&conn.host)
    {
        return Err("Invalid SSH host; use a host without whitespace, control characters, '@', or a leading '-'".to_string());
    }
    args.push("-l".to_string());
    args.push(conn.username.clone());
    args.push(conn.host.clone());
    Ok(())
}

fn play_connecting_animation(nickname: &str, host: &str) {
    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    println!(
        "\x1b[1;36m⚡\x1b[0m Connecting to '\x1b[1;32m{}\x1b[0m' ({})...",
        crate::tui::safe_display(nickname),
        crate::tui::safe_display(host)
    );

    print!("\x1b[?25l");
    let _ = io::stdout().flush();

    let total_steps = 15;
    for i in 0..total_steps {
        let frame = frames[i % frames.len()];
        let status = if i < 5 {
            "Resolving hostname..."
        } else if i < 10 {
            "Negotiating secure port..."
        } else {
            "Passing credentials to SSH..."
        };
        print!("\r  \x1b[1;36m{}\x1b[0m \x1b[2m{}\x1b[0m", frame, status);
        let _ = io::stdout().flush();
        thread::sleep(Duration::from_millis(80));
    }

    print!("\r\x1b[K\x1b[?25h");
    let _ = io::stdout().flush();
}

#[cfg(unix)]
pub fn execute_ssh(conn: &ServerConnection) -> Result<(), String> {
    use std::os::unix::process::CommandExt;

    play_connecting_animation(&conn.nickname, &conn.host);

    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?;

    let mut ssh_args = vec![
        "-p".to_string(),
        conn.port.to_string(),
        "-o".to_string(),
        "ServerAliveInterval=15".to_string(),
        "-o".to_string(),
        "ServerAliveCountMax=3".to_string(),
    ];

    if let Some(ref key) = conn.ssh_key {
        ssh_args.push("-i".to_string());
        ssh_args.push(key.clone());
    }

    append_target_args(&mut ssh_args, conn)?;

    let mut cmd = Command::new("ssh");
    cmd.args(ssh_args);

    let _secret_file = conn
        .password
        .as_deref()
        .map(|p| configure_askpass(&mut cmd, p, &current_exe))
        .transpose()?
        .flatten();

    let err = cmd.exec();
    Err(format!("Failed to execute SSH: {}", err))
}

#[cfg(not(unix))]
pub fn execute_ssh(conn: &ServerConnection) -> Result<(), String> {
    play_connecting_animation(&conn.nickname, &conn.host);

    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?;

    let mut ssh_args = vec![
        "-p".to_string(),
        conn.port.to_string(),
        "-o".to_string(),
        "ServerAliveInterval=15".to_string(),
        "-o".to_string(),
        "ServerAliveCountMax=3".to_string(),
    ];

    if let Some(ref key) = conn.ssh_key {
        ssh_args.push("-i".to_string());
        ssh_args.push(key.clone());
    }

    append_target_args(&mut ssh_args, conn)?;

    let mut cmd = Command::new("ssh");
    cmd.args(ssh_args);

    let _secret_file = conn
        .password
        .as_deref()
        .map(|p| configure_askpass(&mut cmd, p, &current_exe))
        .transpose()?
        .flatten();

    let mut status = cmd
        .status()
        .map_err(|e| format!("Failed to run SSH: {}", e))?;
    if status.success() {
        Ok(())
    } else {
        Err("SSH session exited with an error.".to_string())
    }
}

pub fn execute_ssh_command(conn: &ServerConnection, cmd_str: &str) -> Result<(), String> {
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?;

    let mut ssh_args = vec![
        "-p".to_string(),
        conn.port.to_string(),
        "-o".to_string(),
        "ServerAliveInterval=15".to_string(),
        "-o".to_string(),
        "ServerAliveCountMax=3".to_string(),
    ];

    if let Some(ref key) = conn.ssh_key {
        ssh_args.push("-i".to_string());
        ssh_args.push(key.clone());
    }

    append_target_args(&mut ssh_args, conn)?;
    ssh_args.push(cmd_str.to_string());

    let mut cmd = Command::new("ssh");
    cmd.args(ssh_args);

    let _secret_file = conn
        .password
        .as_deref()
        .map(|p| configure_askpass(&mut cmd, p, &current_exe))
        .transpose()?
        .flatten();

    let status = cmd
        .status()
        .map_err(|e| format!("Failed to run SSH: {}", e))?;
    if status.success() {
        Ok(())
    } else {
        Err("SSH command execution failed.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connection(username: &str, host: &str) -> ServerConnection {
        ServerConnection {
            nickname: "demo".to_string(),
            host: host.to_string(),
            port: 22,
            username: username.to_string(),
            password: None,
            password_ref: None,
            ssh_key: None,
            group: None,
            quick_commands: None,
        }
    }

    #[test]
    fn target_arguments_are_separate_and_reject_option_injection() {
        let mut args = Vec::new();
        append_target_args(&mut args, &connection("deploy", "example.org")).unwrap();
        assert_eq!(args, ["-l", "deploy", "example.org"]);
        assert!(
            append_target_args(&mut Vec::new(), &connection("-oProxyCommand=evil", "host"))
                .is_err()
        );
        assert!(
            append_target_args(&mut Vec::new(), &connection("user", "-oProxyCommand=evil"))
                .is_err()
        );
        assert!(
            append_target_args(&mut Vec::new(), &connection("user", "host\nmalicious")).is_err()
        );
    }
}
