use crate::storage::ServerConnection;
use std::io::{self, Write};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

static ACTIVE_MCP_SSH_COMMANDS: AtomicUsize = AtomicUsize::new(0);
const MAX_ACTIVE_MCP_SSH_COMMANDS: usize = 4;

pub struct McpSshCommandGuard;

impl McpSshCommandGuard {
    pub fn acquire() -> Result<Self, String> {
        ACTIVE_MCP_SSH_COMMANDS
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |active| {
                (active < MAX_ACTIVE_MCP_SSH_COMMANDS).then_some(active + 1)
            })
            .map(|_| Self)
            .map_err(|_| {
                format!(
                    "At most {MAX_ACTIVE_MCP_SSH_COMMANDS} MCP SSH commands may run concurrently"
                )
            })
    }
}

impl Drop for McpSshCommandGuard {
    fn drop(&mut self) {
        ACTIVE_MCP_SSH_COMMANDS.fetch_sub(1, Ordering::AcqRel);
    }
}

#[cfg(unix)]
fn configure_askpass(
    cmd: &mut Command,
    password: &str,
    current_exe: &std::path::Path,
) -> Result<Option<std::fs::File>, String> {
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::OpenOptionsExt;
    use std::os::unix::process::CommandExt;
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
    // Keep close-on-exec in the parent so unrelated concurrent child processes cannot
    // inherit the credential. Clear it only in the specific SSH child immediately
    // before exec; SSH needs it for its askpass helper.
    unsafe {
        cmd.pre_exec(move || {
            let flags = libc::fcntl(fd, libc::F_GETFD);
            if flags == -1 || libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
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
    if conn.port == 0 {
        return Err("Invalid SSH port; use a port between 1 and 65535".to_string());
    }
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

pub struct CapturedCommand {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub output_truncated: bool,
}

/// A foreground SSH ControlMaster owned by one MCP session.
pub struct SshMaster {
    child: std::process::Child,
    dir: std::path::PathBuf,
    socket: std::path::PathBuf,
}

impl SshMaster {
    pub fn socket(&self) -> &std::path::Path {
        &self.socket
    }
}

impl Drop for SshMaster {
    fn drop(&mut self) {
        kill_ssh_process_group(&mut self.child);
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Open a persistent SSH ControlMaster. Its socket is private to this process and
/// the master is terminated when the returned handle is dropped.
pub fn start_ssh_master(
    conn: &ServerConnection,
    timeout_seconds: u64,
) -> Result<SshMaster, String> {
    use std::os::unix::fs::DirBuilderExt;
    use std::os::unix::process::CommandExt;
    use std::process::Stdio;
    use std::time::Instant;

    if !(1..=120).contains(&timeout_seconds) {
        return Err("Timeout must be between 1 and 120 seconds".to_string());
    }
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {e}"))?;
    let dir = std::env::temp_dir().join(format!("termos-{}", uuid::Uuid::new_v4()));
    let mut builder = std::fs::DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(&dir)
        .map_err(|e| format!("Failed to create private SSH session directory: {e}"))?;
    let socket = dir.join("c");

    let mut ssh_args = vec![
        "-M".to_string(),
        "-N".to_string(),
        "-S".to_string(),
        socket.to_string_lossy().into_owned(),
        "-o".to_string(),
        "ControlMaster=yes".to_string(),
        "-o".to_string(),
        "ControlPersist=no".to_string(),
        "-o".to_string(),
        "ServerAliveInterval=15".to_string(),
        "-o".to_string(),
        "ServerAliveCountMax=3".to_string(),
        "-p".to_string(),
        conn.port.to_string(),
    ];
    if let Some(key) = &conn.ssh_key {
        ssh_args.extend(["-i".to_string(), key.clone()]);
    }
    if let Err(error) = append_target_args(&mut ssh_args, conn) {
        let _ = std::fs::remove_dir_all(&dir);
        return Err(error);
    }

    let mut cmd = Command::new("ssh");
    cmd.args(ssh_args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0);
    let mut secret_file = match conn.password.as_deref() {
        Some(password) => match configure_askpass(&mut cmd, password, &current_exe) {
            Ok(file) => file,
            Err(error) => {
                let _ = std::fs::remove_dir_all(&dir);
                return Err(error);
            }
        },
        None => None,
    };
    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&dir);
            return Err(format!("Failed to start SSH master: {error}"));
        }
    };
    let deadline = Instant::now() + Duration::from_secs(timeout_seconds);
    loop {
        if socket.exists() {
            // The master has completed authentication. Remove its inherited credential
            // bytes so a long-lived connection does not retain the password.
            if let Some(mut file) = secret_file.take() {
                use std::io::{Seek, SeekFrom};
                let len = file.metadata().map(|m| m.len()).unwrap_or(0);
                let zeros = vec![0u8; usize::try_from(len).unwrap_or(0).min(65_536)];
                let _ = file.seek(SeekFrom::Start(0));
                let _ = file.write_all(&zeros);
                let _ = file.set_len(0);
                let _ = file.sync_all();
            }
            return Ok(SshMaster { child, dir, socket });
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                let _ = std::fs::remove_dir_all(&dir);
                return Err(format!(
                    "SSH master exited before connecting (status {status})"
                ));
            }
            Err(error) => {
                kill_ssh_process_group(&mut child);
                let _ = child.wait();
                let _ = std::fs::remove_dir_all(&dir);
                return Err(format!("Failed checking SSH master: {error}"));
            }
            Ok(None) => {}
        }
        if Instant::now() >= deadline {
            kill_ssh_process_group(&mut child);
            let _ = child.wait();
            let _ = std::fs::remove_dir_all(&dir);
            return Err("Timed out waiting for SSH master connection".to_string());
        }
        thread::sleep(Duration::from_millis(25));
    }
}

/// Run a non-interactive SSH command and capture bounded output for MCP clients.
pub fn execute_ssh_command_captured_via_session(
    _command_guard: McpSshCommandGuard,
    conn: &ServerConnection,
    cmd_str: &str,
    timeout_seconds: u64,
    control_socket: Option<&std::path::Path>,
) -> Result<CapturedCommand, String> {
    use std::process::Stdio;
    use std::time::Instant;

    const MAX_OUTPUT_PER_STREAM: usize = 32 * 1024;
    if cmd_str.trim().is_empty() {
        return Err("Command must not be empty".to_string());
    }
    if cmd_str.len() > 4096 {
        return Err("Command exceeds the 4096-byte limit".to_string());
    }
    if !(1..=120).contains(&timeout_seconds) {
        return Err("Timeout must be between 1 and 120 seconds".to_string());
    }

    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {e}"))?;
    let mut ssh_args = vec![
        "-p".to_string(),
        conn.port.to_string(),
        "-o".to_string(),
        "ServerAliveInterval=15".to_string(),
        "-o".to_string(),
        "ServerAliveCountMax=3".to_string(),
    ];
    if let Some(socket) = control_socket {
        ssh_args.extend([
            "-S".to_string(),
            socket.to_string_lossy().into_owned(),
            "-o".to_string(),
            "ControlMaster=no".to_string(),
            "-o".to_string(),
            "BatchMode=yes".to_string(),
        ]);
    }
    if let Some(key) = &conn.ssh_key {
        ssh_args.push("-i".to_string());
        ssh_args.push(key.clone());
    }
    append_target_args(&mut ssh_args, conn)?;
    ssh_args.push(cmd_str.to_string());

    let mut cmd = Command::new("ssh");
    cmd.args(ssh_args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let _secret_file = conn
        .password
        .as_deref()
        .map(|p| configure_askpass(&mut cmd, p, &current_exe))
        .transpose()?
        .flatten();

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to start SSH: {e}"))?;
    let stdout = child.stdout.take().ok_or("Could not capture SSH stdout")?;
    let stderr = child.stderr.take().ok_or("Could not capture SSH stderr")?;
    let out_thread = thread::spawn(move || read_bounded(stdout, MAX_OUTPUT_PER_STREAM));
    let err_thread = thread::spawn(move || read_bounded(stderr, MAX_OUTPUT_PER_STREAM));

    let deadline = Instant::now() + Duration::from_secs(timeout_seconds);
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|e| format!("Failed waiting for SSH: {e}"))?
        {
            break Some(status);
        }
        if Instant::now() >= deadline {
            timed_out = true;
            kill_ssh_process_group(&mut child);
            break child.wait().ok();
        }
        thread::sleep(Duration::from_millis(25));
    };

    let (stdout, stdout_was_truncated) = out_thread
        .join()
        .map_err(|_| "SSH stdout reader failed".to_string())??;
    let (stderr, stderr_was_truncated) = err_thread
        .join()
        .map_err(|_| "SSH stderr reader failed".to_string())??;
    let (stdout, stdout_decode_truncated) = sanitize_output_bounded(&stdout, MAX_OUTPUT_PER_STREAM);
    let (stderr, stderr_decode_truncated) = sanitize_output_bounded(&stderr, MAX_OUTPUT_PER_STREAM);

    Ok(CapturedCommand {
        stdout,
        stderr,
        exit_code: status.and_then(|s| s.code()),
        timed_out,
        output_truncated: stdout_was_truncated
            || stderr_was_truncated
            || stdout_decode_truncated
            || stderr_decode_truncated,
    })
}

fn kill_ssh_process_group(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        if let Ok(process_group) = libc::pid_t::try_from(child.id()) {
            // The SSH process starts in a dedicated group so its local ProxyCommand children
            // cannot keep captured output pipes open after the command timeout.
            unsafe {
                libc::kill(-process_group, libc::SIGKILL);
            }
        }
    }
    let _ = child.kill();
}

fn sanitize_output_bounded(bytes: &[u8], limit: usize) -> (String, bool) {
    let decoded = String::from_utf8_lossy(bytes);
    let mut output = String::with_capacity(limit.min(decoded.len()));
    let mut truncated = false;
    for ch in decoded.chars() {
        let bidi_override = matches!(
            ch as u32,
            0x061C | 0x200E | 0x200F | 0x202A..=0x202E | 0x2066..=0x2069
        );
        let mut encoded = [0; 4];
        let escaped;
        let text = if (ch.is_control() && !matches!(ch, '\n' | '\t')) || bidi_override {
            escaped = ch.escape_unicode().to_string();
            escaped.as_str()
        } else {
            ch.encode_utf8(&mut encoded)
        };
        if output.len() + text.len() > limit {
            truncated = true;
            break;
        }
        output.push_str(text);
    }
    (output, truncated)
}

fn read_bounded<R: std::io::Read>(mut reader: R, limit: usize) -> Result<(Vec<u8>, bool), String> {
    let mut result = Vec::with_capacity(limit);
    let mut buffer = [0; 4096];
    let mut truncated = false;
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|e| format!("Failed reading SSH output: {e}"))?;
        if read == 0 {
            break;
        }
        let room = limit.saturating_sub(result.len());
        let keep = room.min(read);
        result.extend_from_slice(&buffer[..keep]);
        truncated |= keep < read;
    }
    Ok((result, truncated))
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
        let mut invalid_port = connection("deploy", "example.org");
        invalid_port.port = 0;
        assert!(append_target_args(&mut Vec::new(), &invalid_port).is_err());
    }

    #[test]
    fn captured_output_is_limited_after_utf8_replacement() {
        let (output, truncated) = sanitize_output_bounded(&[0xff; 32], 24);
        assert!(truncated);
        assert!(output.len() <= 24);
        assert!(output.is_char_boundary(output.len()));
    }

    #[test]
    fn captured_output_escapes_terminal_and_bidi_controls() {
        let (output, truncated) = sanitize_output_bounded(b"ok\x1b[2J\n\xE2\x80\xAE", 64);
        assert_eq!(output, "ok\\u{1b}[2J\n\\u{202e}");
        assert!(!truncated);
    }

    #[test]
    fn captured_output_reader_drains_and_caps_stream() {
        let input = vec![b'x'; 64];
        let (output, truncated) = read_bounded(input.as_slice(), 16).unwrap();
        assert_eq!(output, vec![b'x'; 16]);
        assert!(truncated);
    }

    #[test]
    fn mcp_ssh_concurrency_limit_rejects_excess_calls() {
        let guards = (0..MAX_ACTIVE_MCP_SSH_COMMANDS)
            .map(|_| McpSshCommandGuard::acquire().unwrap())
            .collect::<Vec<_>>();
        assert!(McpSshCommandGuard::acquire().is_err());
        drop(guards);
        assert!(McpSshCommandGuard::acquire().is_ok());
    }
}
