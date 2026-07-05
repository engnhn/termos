mod storage;
mod tui;
mod ssh;

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
    /// Manage quick commands for a server
    #[command(name = "quick-command", aliases = ["qc"])]
    QuickCommand {
        #[command(subcommand)]
        subcommand: QcCommands,
    },
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
    if let Ok(password) = std::env::var("TERMOS_ASKPASS_PASSWORD") {
        println!("{}", password);
        return;
    }

    let cli = Cli::parse();

    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            return;
        }
    };

    match command {
        Commands::Add => {
            match tui::run_wizard(None) {
                Ok(Some(conn)) => {
                    let nickname = conn.nickname.clone();
                    if let Err(e) = add_connection(conn) {
                        eprintln!("\n\x1b[1;31mError:\x1b[0m {}", e);
                    } else {
                        println!("\n\x1b[1;32m✔\x1b[0m Server '{}' saved successfully!", nickname);
                    }
                }
                Ok(None) => {
                    println!("\n\x1b[1;33mCancelled.\x1b[0m No server was saved.");
                }
                Err(e) => {
                    eprintln!("\n\x1b[1;31mWizard Error:\x1b[0m {}", e);
                }
            }
        }
        Commands::Connect { nickname, qc } => {
            match get_connection(&nickname) {
                Ok(Some(conn)) => {
                    if let Some(qc_name) = qc {
                        if let Some(ref cmds) = conn.quick_commands {
                            if let Some(target_cmd) = cmds.iter().find(|c| c.name.eq_ignore_ascii_case(&qc_name)) {
                                println!("\x1b[1;36m⚡ Executing '{}' on {}...\x1b[0m", target_cmd.name, conn.nickname);
                                println!("\x1b[1;30mCommand: {}\x1b[0m\n", target_cmd.command);
                                if let Err(e) = ssh::execute_ssh_command(&conn, &target_cmd.command) {
                                    eprintln!("\n\x1b[1;31mError:\x1b[0m {}", e);
                                    std::process::exit(1);
                                }
                            } else {
                                eprintln!("\x1b[1;31mError:\x1b[0m Quick command '{}' not found for server '{}'.", qc_name, nickname);
                                std::process::exit(1);
                            }
                        } else {
                            eprintln!("\x1b[1;31mError:\x1b[0m Server '{}' has no quick commands defined.", nickname);
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
                    eprintln!("\x1b[1;31mError:\x1b[0m Server with nickname '{}' not found. Use 'termos list' to view registered servers.", nickname);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("\x1b[1;31mDatabase Error:\x1b[0m {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::List => {
            match tui::run_list_manager() {
                Ok(Some(conn)) => {
                    if let Err(e) = ssh::execute_ssh(&conn) {
                        eprintln!("\x1b[1;31mConnection Failed:\x1b[0m {}", e);
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
                }
            }
        }
        Commands::Delete { nickname } => {
            match delete_connection(&nickname) {
                Ok(true) => {
                    println!("\x1b[1;32m✔\x1b[0m Server '{}' deleted successfully.", nickname);
                }
                Ok(false) => {
                    eprintln!("\x1b[1;31mError:\x1b[0m Server with nickname '{}' not found.", nickname);
                }
                Err(e) => {
                    eprintln!("\x1b[1;31mDatabase Error:\x1b[0m {}", e);
                }
            }
        }
        Commands::Update => {
            println!("\x1b[1;36m⚡ Fetching and installing the latest version of Termos from GitHub...\x1b[0m");
            let mut cmd = std::process::Command::new("bash");
            cmd.arg("-c").arg("curl -fsSL https://raw.githubusercontent.com/engnhn/termos/main/install.sh | bash");
            match cmd.status() {
                Ok(status) if status.success() => {
                    println!("\x1b[1;32m✔ Termos updated successfully!\x1b[0m");
                }
                _ => {
                    eprintln!("\x1b[1;31mError:\x1b[0m Failed to update Termos. Please verify curl and internet access.");
                }
            }
        }
        Commands::QuickCommand { subcommand } => {
            match subcommand {
                QcCommands::List { nickname } => {
                    if let Err(e) = tui::run_qc_manager(nickname, tui::QcMode::List) {
                        eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
                    }
                }
                QcCommands::Add { nickname, name, cmd } => {
                    if nickname.is_some() && name.is_some() && cmd.is_some() {
                        let nickname = nickname.unwrap();
                        let name = name.unwrap();
                        let cmd = cmd.unwrap();
                        let qc = crate::storage::QuickCommand { name, command: cmd };
                        match crate::storage::add_quick_command(&nickname, qc) {
                            Ok(_) => {
                                println!("\x1b[1;32m✔\x1b[0m Quick command added successfully.");
                            }
                            Err(e) => {
                                eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
                            }
                        }
                    } else {
                        if let Err(e) = tui::run_qc_manager(nickname, tui::QcMode::Add) {
                            eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
                        }
                    }
                }
                QcCommands::Edit { nickname, name, new_name, new_cmd } => {
                    if nickname.is_some() && name.is_some() {
                        let nickname = nickname.unwrap();
                        let name = name.unwrap();
                        if new_name.is_none() && new_cmd.is_none() {
                            eprintln!("\x1b[1;31mError:\x1b[0m Provide either --new-name or --new-cmd to edit.");
                            return;
                        }
                        match crate::storage::edit_quick_command(&nickname, &name, new_name, new_cmd) {
                            Ok(_) => {
                                println!("\x1b[1;32m✔\x1b[0m Quick command updated successfully.");
                            }
                            Err(e) => {
                                eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
                            }
                        }
                    } else {
                        if let Err(e) = tui::run_qc_manager(nickname, tui::QcMode::Edit) {
                            eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
                        }
                    }
                }
                QcCommands::Delete { nickname, name } => {
                    if nickname.is_some() && name.is_some() {
                        let nickname = nickname.unwrap();
                        let name = name.unwrap();
                        match crate::storage::delete_quick_command(&nickname, &name) {
                            Ok(_) => {
                                println!("\x1b[1;32m✔\x1b[0m Quick command deleted successfully.");
                            }
                            Err(e) => {
                                eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
                            }
                        }
                    } else {
                        if let Err(e) = tui::run_qc_manager(nickname, tui::QcMode::Delete) {
                            eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
                        }
                    }
                }
            }
        }
        Commands::_ListNicknames => {
            if let Ok(conns) = load_connections() {
                for conn in conns {
                    println!("{}", conn.nickname);
                }
            }
        }
    }
}
