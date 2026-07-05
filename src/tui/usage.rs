use std::io::{stdout, Write};
use crossterm::{
    event::{self, Event, KeyCode},
    QueueableCommand,
    style::{Color, SetForegroundColor, ResetColor, SetAttribute, Attribute},
    terminal::{Clear, ClearType},
};
use super::common::{TerminalGuard, check_size_or_draw_error, draw_box};

pub fn run_usage_viewer() -> Result<(), String> {
    let mut _guard = TerminalGuard::create()?;
    let mut out = stdout();

    let box_width = 76;
    let box_height = 20;
    let mut current_page = 0;
    let total_pages = 3;

    loop {
        out.queue(Clear(ClearType::All)).unwrap();

        let (cols, rows) = match check_size_or_draw_error(&mut out, box_width, box_height) {
            Ok(Some(sz)) => sz,
            _ => {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                        break;
                    }
                }
                continue;
            }
        };

        let start_x = (cols.saturating_sub(box_width)) / 2;
        let start_y = (rows.saturating_sub(box_height)) / 2;

        let title = format!(" USER MANUAL (PAGE {} OF {}) ", current_page + 1, total_pages);
        draw_box(&mut out, start_x, start_y, box_width, box_height, &title);

        out.queue(SetForegroundColor(Color::White)).unwrap();
        match current_page {
            0 => {
                out.queue(crossterm::cursor::MoveTo(start_x + 4, start_y + 2)).unwrap();
                out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                out.queue(SetAttribute(Attribute::Bold)).unwrap();
                print!("1. INTERACTIVE TUI DASHBOARD & NAVIGATION");
                out.queue(ResetColor).unwrap();
                out.queue(SetAttribute(Attribute::Reset)).unwrap();
                out.queue(SetForegroundColor(Color::White)).unwrap();

                let lines = [
                    "• Launch the dashboard via: 'termos' or 'termos list'",
                    "• [Up/Down Arrows] : Select server configuration.",
                    "• [Enter]          : Initiate interactive SSH terminal connection.",
                    "• [a]              : Inline wizard to add new server configurations.",
                    "• [d] or [Delete]  : Safely remove server connection profile.",
                    "• [g] or [/]        : Toggle Group/Namespace selection popover.",
                    "",
                    "DASHBOARD OVERLAY MANAGEMENT ROUTING:",
                    "• [e] (Manage)     : Edit server fields or route to QC Manager.",
                    "• [c] (Quick Cmd)  : Opens overlay to execute pre-saved scripts.",
                ];

                for (idx, line) in lines.iter().enumerate() {
                    out.queue(crossterm::cursor::MoveTo(start_x + 4, start_y + 4 + idx as u16)).unwrap();
                    print!("{}", line);
                }
            }
            1 => {
                out.queue(crossterm::cursor::MoveTo(start_x + 4, start_y + 2)).unwrap();
                out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                out.queue(SetAttribute(Attribute::Bold)).unwrap();
                print!("2. QUICK COMMANDS (QC) ORCHESTRATION");
                out.queue(ResetColor).unwrap();
                out.queue(SetAttribute(Attribute::Reset)).unwrap();
                out.queue(SetForegroundColor(Color::White)).unwrap();

                let lines = [
                    "CREATING & MANAGING SCRIPTS:",
                    "• Press [e] on a server -> Select Option 2: Manage QCs.",
                    "• Press [a] to Add, [e] to Edit, [d] to Delete within the TUI manager.",
                    "• Alternatively, manage programmatically via CLI subcommands:",
                    "  termos qc [list | add | edit | delete] [nickname] [options]",
                    "",
                    "REMOTE SCRIPT EXECUTION:",
                    "• Interactive TUI: Press [c] on any server, choose command, hit [Enter].",
                    "• Direct CLI Shortcut: Run commands directly from your native shell:",
                    "  termos connect <nickname> -q <qc_name>",
                ];

                for (idx, line) in lines.iter().enumerate() {
                    out.queue(crossterm::cursor::MoveTo(start_x + 4, start_y + 4 + idx as u16)).unwrap();
                    print!("{}", line);
                }
            }
            2 => {
                out.queue(crossterm::cursor::MoveTo(start_x + 4, start_y + 2)).unwrap();
                out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                out.queue(SetAttribute(Attribute::Bold)).unwrap();
                print!("3. SECURITY CONTROLS & AUTOCOMPLETION");
                out.queue(ResetColor).unwrap();
                out.queue(SetAttribute(Attribute::Reset)).unwrap();
                out.queue(SetForegroundColor(Color::White)).unwrap();

                let lines = [
                    "SYSTEM BOUNDARY & CREDENTIAL ISOLATION:",
                    "• RAII TerminalGuard guarantees raw-mode resets on crashes or exits.",
                    "• Passwords are piped to SSH client via secure SSH_ASKPASS handlers.",
                    "  Prevents password exposure in user-inspectable system process trees.",
                    "• Configurations stored under ~/.config/termos/connections.json",
                    "  Protected with 0600 user-only permissions.",
                    "",
                    "SHELL TAB COMPLETIONS:",
                    "• Automatic completions are registered for Bash and Zsh.",
                    "• Tab suggests nicknames, command routes, and option --flags.",
                ];

                for (idx, line) in lines.iter().enumerate() {
                    out.queue(crossterm::cursor::MoveTo(start_x + 4, start_y + 4 + idx as u16)).unwrap();
                    print!("{}", line);
                }
            }
            _ => {}
        }

        let div_y = start_y + box_height - 5;
        let help_line = "Page: [Left/Right] or [Tab]  |  Exit: [Esc] or [q]";
        let h_x = start_x + box_width.saturating_sub(help_line.chars().count() as u16) / 2;
        out.queue(crossterm::cursor::MoveTo(h_x, div_y + 2)).unwrap();
        out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
        print!("{}", help_line);
        out.queue(ResetColor).unwrap();

        out.flush().unwrap();

        if let Ok(Event::Key(key)) = event::read() {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                    break;
                }
                KeyCode::Left | KeyCode::Up => {
                    current_page = if current_page == 0 { total_pages - 1 } else { current_page - 1 };
                }
                KeyCode::Right | KeyCode::Down | KeyCode::Tab => {
                    current_page = (current_page + 1) % total_pages;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
