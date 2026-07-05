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
    },
    /// List all registered servers
    List,
    /// Delete a registered server connection
    Delete {
        /// Nickname of the server to delete
        nickname: String,
    },
    /// Hidden subcommand for shell tab completion listing nicknames
    #[command(name = "_list-nicknames", hide = true)]
    _ListNicknames,
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
            match tui::run_add_wizard() {
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
        Commands::Connect { nickname } => {
            match get_connection(&nickname) {
                Ok(Some(conn)) => {
                    if let Err(e) = ssh::execute_ssh(&conn) {
                        eprintln!("\x1b[1;31mConnection Failed:\x1b[0m {}", e);
                    }
                }
                Ok(None) => {
                    eprintln!("\x1b[1;31mError:\x1b[0m Server with nickname '{}' not found. Use 'termos list' to view registered servers.", nickname);
                }
                Err(e) => {
                    eprintln!("\x1b[1;31mDatabase Error:\x1b[0m {}", e);
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
        Commands::_ListNicknames => {
            if let Ok(conns) = load_connections() {
                for conn in conns {
                    println!("{}", conn.nickname);
                }
            }
        }
    }
}
