use crate::storage::ServerConnection;
use std::process::Command;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn play_connecting_animation(nickname: &str, host: &str) {
    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    println!("\x1b[1;36m⚡\x1b[0m Connecting to '\x1b[1;32m{}\x1b[0m' ({})...", nickname, host);
    
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

    ssh_args.push(format!("{}@{}", conn.username, conn.host));

    let mut cmd = Command::new("ssh");
    cmd.args(ssh_args);

    if let Some(ref password) = conn.password {
        cmd.env("SSH_ASKPASS", &current_exe);
        cmd.env("SSH_ASKPASS_REQUIRE", "force");
        cmd.env("TERMOS_ASKPASS_PASSWORD", password);
        
        if std::env::var("DISPLAY").is_err() {
            cmd.env("DISPLAY", ":0");
        }
    }

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

    ssh_args.push(format!("{}@{}", conn.username, conn.host));

    let mut cmd = Command::new("ssh");
    cmd.args(ssh_args);

    if let Some(ref password) = conn.password {
        cmd.env("SSH_ASKPASS", &current_exe);
        cmd.env("SSH_ASKPASS_REQUIRE", "force");
        cmd.env("TERMOS_ASKPASS_PASSWORD", password);
        if std::env::var("DISPLAY").is_err() {
            cmd.env("DISPLAY", ":0");
        }
    }

    let mut status = cmd.status().map_err(|e| format!("Failed to run SSH: {}", e))?;
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

    ssh_args.push(format!("{}@{}", conn.username, conn.host));
    ssh_args.push(cmd_str.to_string());

    let mut cmd = Command::new("ssh");
    cmd.args(ssh_args);

    if let Some(ref password) = conn.password {
        cmd.env("SSH_ASKPASS", &current_exe);
        cmd.env("SSH_ASKPASS_REQUIRE", "force");
        cmd.env("TERMOS_ASKPASS_PASSWORD", password);
        if std::env::var("DISPLAY").is_err() {
            cmd.env("DISPLAY", ":0");
        }
    }

    let status = cmd.status().map_err(|e| format!("Failed to run SSH: {}", e))?;
    if status.success() {
        Ok(())
    } else {
        Err("SSH command execution failed.".to_string())
    }
}
