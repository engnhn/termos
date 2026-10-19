mod mcp_server;
mod ssh;
mod storage;
mod tui;
mod update;

use clap::{Parser, Subcommand};
use storage::{add_connection, delete_connection, get_connection, load_connections};

#[derive(Parser)]
#[command(name = "termos")]
#[command(version = "0.1.0")]
#[command(about = "Termos - Local connection manager for virtual servers", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new server connection (Interactive Wizard)
    Add,
    /// Establish an SSH connection to a saved server
    Connect {
        /// Nickname of the server to connect to
        nickname: String,
        /// Optional quick command name to run immediately instead of interactive shell
        #[arg(long, short = 'q')]
        qc: Option<String>,
    },
    /// List all registered servers
    List,
    /// Delete a registered server connection
    Delete {
        /// Nickname of the server to delete
        nickname: String,
    },
    /// Update Termos to the latest version directly from GitHub
    Update,
    /// Run a local MCP server over stdio
    Mcp {
        /// Expose the tool that executes remote shell commands on saved servers
        #[arg(long)]
        allow_remote_commands: bool,
    },
    /// Manage quick commands for a server
    #[command(name = "quick-command", aliases = ["qc"])]
    QuickCommand {
        #[command(subcommand)]
        subcommand: QcCommands,
    },
    /// View detailed interactive user manual and usage guide
    Usage,
    /// Hidden subcommand for shell tab completion listing nicknames
    #[command(name = "_list-nicknames", hide = true)]
    _ListNicknames,
}

#[derive(Subcommand)]
pub enum QcCommands {
    /// List quick commands (interactive TUI if nickname omitted)
    List {
        /// Nickname of the server (optional)
        nickname: Option<String>,
    },
    /// Add a quick command (interactive TUI if parameters omitted)
    Add {
        /// Nickname of the server (optional)
        nickname: Option<String>,
        /// Name of the quick command (optional for interactive TUI)
        #[arg(long)]
        name: Option<String>,
        /// The SSH command to execute (optional for interactive TUI)
        #[arg(long)]
        cmd: Option<String>,
    },
    /// Edit an existing quick command (interactive TUI if parameters omitted)
    Edit {
        /// Nickname of the server (optional)
        nickname: Option<String>,
        /// Current name of the quick command (optional for interactive TUI)
        #[arg(long)]
        name: Option<String>,
        /// New name of the quick command (optional)
        #[arg(long)]
        new_name: Option<String>,
        /// New SSH command string (optional)
        #[arg(long)]
        new_cmd: Option<String>,
    },
    /// Delete a quick command (interactive TUI if nickname/name omitted)
    Delete {
        /// Nickname of the server (optional)
        nickname: Option<String>,
        /// Name of the quick command to delete (optional for interactive TUI)
        #[arg(long)]
        name: Option<String>,
    },
}

fn main() {
    #[cfg(unix)]
    if let (Ok(fd), Ok(len)) = (
        std::env::var("TERMOS_ASKPASS_FD"),
        std::env::var("TERMOS_ASKPASS_LEN"),
    ) {
        use std::io::{Read, Seek, SeekFrom};
        use std::os::fd::FromRawFd;

        let is_askpass = std::env::var_os("SSH_ASKPASS")
            .zip(std::env::current_exe().ok())
            .is_some_and(|(configured, current)| std::path::Path::new(&configured) == current);
        if is_askpass
            && let (Ok(fd), Ok(len)) = (fd.parse::<i32>(), len.parse::<usize>())
            && (3..=libc::c_int::MAX).contains(&fd)
            && len > 0
            && len <= 65_536
        {
            let mut stat: libc::stat = unsafe { std::mem::zeroed() };
            let valid_file = unsafe { libc::fstat(fd, &mut stat) } == 0
                && stat.st_mode & libc::S_IFMT == libc::S_IFREG
                && stat.st_size >= len as libc::off_t;
            if valid_file {
                // The descriptor is an unlinked, mode-0600 file created by ssh.rs.
                let mut secret_file = unsafe { std::fs::File::from_raw_fd(fd) };
                let mut password = vec![0; len];
                if secret_file.seek(SeekFrom::Start(0)).is_ok()
                    && secret_file.read_exact(&mut password).is_ok()
                {
                    use std::io::Write;
                    let _ = std::io::stdout().write_all(&password);
                    let _ = std::io::stdout().write_all(b"\n");
                }
                zeroize::Zeroize::zeroize(&mut password);
                return;
            }
        }
    }

    let cli = Cli::parse();

    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            match tui::run_list_manager() {
                Ok(Some(conn)) => {
                    if let Err(e) = ssh::execute_ssh(&conn) {
                        eprintln!("\n\x1b[1;31mConnection Failed:\x1b[0m {}", e);
                        std::process::exit(1);
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("\n\x1b[1;31mError:\x1b[0m {}", e);
                    std::process::exit(1);
                }
            }
            return;
        }
    };

    match command {
        Commands::Add => match tui::run_wizard(None) {
            Ok(Some(conn)) => {
                let nickname = conn.nickname.clone();
                if let Err(e) = add_connection(conn) {
                    eprintln!("\n\x1b[1;31mError:\x1b[0m {}", e);
                    std::process::exit(1);
                } else {
                    println!(
                        "\n\x1b[1;32m✔\x1b[0m Server '{}' saved successfully!",
                        tui::safe_display(&nickname)
                    );
                }
            }
            Ok(None) => {
                println!("\n\x1b[1;33mCancelled.\x1b[0m No server was saved.");
            }
            Err(e) => {
                eprintln!("\n\x1b[1;31mWizard Error:\x1b[0m {}", e);
                std::process::exit(1);
            }
        },
        Commands::Connect { nickname, qc } => match get_connection(&nickname) {
            Ok(Some(conn)) => {
                if let Some(qc_name) = qc {
                    if let Some(ref cmds) = conn.quick_commands {
                        if let Some(target_cmd) =
                            cmds.iter().find(|c| c.name.eq_ignore_ascii_case(&qc_name))
                        {
                            println!(
                                "\x1b[1;36m⚡ Executing '{}' on {}...\x1b[0m",
                                tui::safe_display(&target_cmd.name),
                                tui::safe_display(&conn.nickname)
                            );
                            println!(
                                "\x1b[1;30mCommand: {}\x1b[0m\n",
                                tui::safe_display(&target_cmd.command)
                            );
                            if let Err(e) = ssh::execute_ssh_command(&conn, &target_cmd.command) {
                                eprintln!("\n\x1b[1;31mError:\x1b[0m {}", e);
                                std::process::exit(1);
                            }
                        } else {
                            eprintln!(
                                "\x1b[1;31mError:\x1b[0m Quick command '{}' not found for server '{}'.",
                                qc_name, nickname
                            );
                            std::process::exit(1);
                        }
                    } else {
                        eprintln!(
                            "\x1b[1;31mError:\x1b[0m Server '{}' has no quick commands defined.",
                            nickname
                        );
                        std::process::exit(1);
                    }
                } else {
                    if let Err(e) = ssh::execute_ssh(&conn) {
                        eprintln!("\x1b[1;31mConnection Failed:\x1b[0m {}", e);
                        std::process::exit(1);
                    }
                }
            }
            Ok(None) => {
                eprintln!(
                    "\x1b[1;31mError:\x1b[0m Server with nickname '{}' not found. Use 'termos list' to view registered servers.",
                    nickname
                );
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("\x1b[1;31mDatabase Error:\x1b[0m {}", e);
                std::process::exit(1);
            }
        },
        Commands::List => match tui::run_list_manager() {
            Ok(Some(conn)) => {
                if let Err(e) = ssh::execute_ssh(&conn) {
                    eprintln!("\x1b[1;31mConnection Failed:\x1b[0m {}", e);
                    std::process::exit(1);
                }
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
                std::process::exit(1);
            }
        },
        Commands::Delete { nickname } => match delete_connection(&nickname) {
            Ok(true) => {
                println!(
                    "\x1b[1;32m✔\x1b[0m Server '{}' deleted successfully.",
                    tui::safe_display(&nickname)
                );
            }
            Ok(false) => {
                eprintln!(
                    "\x1b[1;31mError:\x1b[0m Server with nickname '{}' not found.",
                    tui::safe_display(&nickname)
                );
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("\x1b[1;31mDatabase Error:\x1b[0m {}", e);
                std::process::exit(1);
            }
        },
        Commands::Update => {
            println!(
                "\x1b[1;36m⚡ Downloading and verifying the latest signed Termos release...\x1b[0m"
            );
            match update::update() {
                Ok(()) => {
                    println!("\x1b[1;32m✔ Termos updated successfully!\x1b[0m");
                }
                Err(e) => {
                    eprintln!("\x1b[1;31mUpdate failed:\x1b[0m {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Mcp {
            allow_remote_commands,
        } => {
            let runtime = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    eprintln!("Failed to start MCP runtime: {error}");
                    std::process::exit(1);
                }
            };
            if let Err(error) = runtime.block_on(mcp_server::run(allow_remote_commands)) {
                eprintln!("MCP server stopped: {error}");
                std::process::exit(1);
            }
        }
        Commands::QuickCommand { subcommand } => match subcommand {
            QcCommands::List { nickname } => {
                if let Err(e) = tui::run_qc_manager(nickname, tui::QcMode::List) {
                    eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
                    std::process::exit(1);
                }
            }
            QcCommands::Add {
                nickname,
                name,
                cmd,
            } => match (nickname, name, cmd) {
                (Some(nickname), Some(name), Some(cmd)) => {
                    let qc = crate::storage::QuickCommand { name, command: cmd };
                    match crate::storage::add_quick_command(&nickname, qc) {
                        Ok(_) => {
                            println!("\x1b[1;32m✔\x1b[0m Quick command added successfully.");
                        }
                        Err(e) => {
                            eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                (nickname, _, _) => {
                    if let Err(e) = tui::run_qc_manager(nickname, tui::QcMode::Add) {
                        eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
                        std::process::exit(1);
                    }
                }
            },
            QcCommands::Edit {
                nickname,
                name,
                new_name,
                new_cmd,
            } => match (nickname, name) {
                (Some(nickname), Some(name)) => {
                    if new_name.is_none() && new_cmd.is_none() {
                        eprintln!(
                            "\x1b[1;31mError:\x1b[0m Provide either --new-name or --new-cmd to edit."
                        );
                        std::process::exit(2);
                    }
                    match crate::storage::edit_quick_command(&nickname, &name, new_name, new_cmd) {
                        Ok(_) => {
                            println!("\x1b[1;32m✔\x1b[0m Quick command updated successfully.");
                        }
                        Err(e) => {
                            eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                (nickname, _) => {
                    if let Err(e) = tui::run_qc_manager(nickname, tui::QcMode::Edit) {
                        eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
                        std::process::exit(1);
                    }
                }
            },
            QcCommands::Delete { nickname, name } => match (nickname, name) {
                (Some(nickname), Some(name)) => {
                    match crate::storage::delete_quick_command(&nickname, &name) {
                        Ok(_) => {
                            println!("\x1b[1;32m✔\x1b[0m Quick command deleted successfully.");
                        }
                        Err(e) => {
                            eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                (nickname, _) => {
                    if let Err(e) = tui::run_qc_manager(nickname, tui::QcMode::Delete) {
                        eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
                        std::process::exit(1);
                    }
                }
            },
        },
        Commands::Usage => {
            if let Err(e) = tui::run_usage_viewer() {
                eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
                std::process::exit(1);
            }
        }
        Commands::_ListNicknames => {
            if let Ok(conns) = load_connections() {
                for conn in conns {
                    println!("{}", tui::safe_display(&conn.nickname));
                }
            }
        }
    }
}
