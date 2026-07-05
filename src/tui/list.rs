use crate::storage::ServerConnection;
use super::common::{draw_box, check_size_or_draw_error, TerminalGuard};
use super::wizard::run_add_wizard;
use crossterm::{
    event::{self, Event, KeyCode},
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use std::io::{stdout, Write};

pub fn run_list_manager() -> Result<Option<ServerConnection>, String> {
    let mut _guard = TerminalGuard::create()?;
    let mut out = stdout();

    let mut selected_idx = 0;
    let mut scroll_offset = 0;
    let mut confirm_delete = false;
    let mut status_msg: Option<String> = None;
    let box_width = 66;
    let box_height = 18;

    let result = loop {
        let conns = match crate::storage::load_connections() {
            Ok(list) => list,
            Err(e) => {
                break Err(format!("Database error: {}", e));
            }
        };

        if !conns.is_empty() && selected_idx >= conns.len() {
            selected_idx = conns.len() - 1;
        }

        if selected_idx < scroll_offset {
            scroll_offset = selected_idx;
        } else if !conns.is_empty() && selected_idx >= scroll_offset + 8 {
            scroll_offset = selected_idx - 7;
        }

        out.queue(Clear(ClearType::All)).unwrap();

        let (cols, rows) = match check_size_or_draw_error(&mut out, box_width, box_height) {
            Ok(Some(sz)) => sz,
            _ => {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.code == KeyCode::Esc {
                        break Ok(None);
                    }
                }
                continue;
            }
        };

        let start_x = (cols.saturating_sub(box_width)) / 2;
        let start_y = (rows.saturating_sub(box_height)) / 2;

        draw_box(&mut out, start_x, start_y, box_width, box_height, " TERMOS - CONNECTIONS ");

        if conns.is_empty() {
            let empty_msg = "No servers registered yet.";
            let x = start_x + (box_width - empty_msg.chars().count() as u16) / 2;
            out.queue(crossterm::cursor::MoveTo(x, start_y + 5)).unwrap();
            out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
            print!("{}", empty_msg);

            let add_hint = "Press [a] to add a new server or [Esc] to exit.";
            let x2 = start_x + (box_width - add_hint.chars().count() as u16) / 2;
            out.queue(crossterm::cursor::MoveTo(x2, start_y + 7)).unwrap();
            print!("{}", add_hint);
            out.queue(ResetColor).unwrap();
        } else {
            for i in 0..8 {
                let idx = scroll_offset + i;
                if idx >= conns.len() {
                    break;
                }

                let conn = &conns[idx];
                let is_selected = idx == selected_idx;
                let row_y = start_y + 2 + i as u16;

                out.queue(crossterm::cursor::MoveTo(start_x + 3, row_y)).unwrap();

                let key_tag = if conn.ssh_key.is_some() { " [Key]" } else { "" };
                let display_str = format!("{:<16} ({}){}", conn.nickname, format!("{}@{}:{}", conn.username, conn.host, conn.port), key_tag);
                let truncated_row: String = display_str.chars().take((box_width - 8) as usize).collect();

                if is_selected {
                    out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                    out.queue(SetAttribute(Attribute::Bold)).unwrap();
                    print!("▶ ");
                    out.queue(SetForegroundColor(Color::White)).unwrap();
                    print!("{:<width$}", truncated_row, width = (box_width - 8) as usize);
                } else {
                    print!("  ");
                    out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                    print!("{:<width$}", truncated_row, width = (box_width - 8) as usize);
                }
                out.queue(ResetColor).unwrap();
                out.queue(SetAttribute(Attribute::Reset)).unwrap();
            }

            if scroll_offset > 0 {
                out.queue(crossterm::cursor::MoveTo(start_x + box_width - 4, start_y + 2)).unwrap();
                out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                print!("▲");
            }
            if scroll_offset + 8 < conns.len() {
                out.queue(crossterm::cursor::MoveTo(start_x + box_width - 4, start_y + 9)).unwrap();
                out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                print!("▼");
            }
            out.queue(ResetColor).unwrap();
        }

        let div_y = start_y + box_height - 5;
        
        if confirm_delete && !conns.is_empty() {
            let active_conn = &conns[selected_idx];
            let confirm_line = format!("Delete '{}'? (y/n)", active_conn.nickname);
            let confirm_x = start_x + (box_width - confirm_line.chars().count() as u16) / 2;
            out.queue(crossterm::cursor::MoveTo(confirm_x, div_y + 1)).unwrap();
            out.queue(SetForegroundColor(Color::Black)).unwrap();
            out.queue(crossterm::style::SetBackgroundColor(Color::Red)).unwrap();
            out.queue(SetAttribute(Attribute::Bold)).unwrap();
            print!(" {} ", confirm_line);
            out.queue(ResetColor).unwrap();
            out.queue(crossterm::style::SetBackgroundColor(Color::Reset)).unwrap();
            out.queue(SetAttribute(Attribute::Reset)).unwrap();
        } else {
            if let Some(ref status) = status_msg {
                let status_x = start_x + (box_width - status.chars().count() as u16) / 2;
                out.queue(crossterm::cursor::MoveTo(status_x, div_y + 1)).unwrap();
                out.queue(SetForegroundColor(Color::Green)).unwrap();
                out.queue(SetAttribute(Attribute::Bold)).unwrap();
                print!("{}", status);
                out.queue(ResetColor).unwrap();
                out.queue(SetAttribute(Attribute::Reset)).unwrap();
            } else {
                let help_l1 = "Navigate: [Up/Down] arrows  |  Connect: [Enter]";
                let help_l2 = "Actions:  [a] Add Server  |  [d] Delete Server  |  [ESC] Exit";

                let h1_x = start_x + (box_width - help_l1.chars().count() as u16) / 2;
                let h2_x = start_x + (box_width - help_l2.chars().count() as u16) / 2;

                out.queue(crossterm::cursor::MoveTo(h1_x, div_y + 1)).unwrap();
                out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                print!("{}", help_l1);

                out.queue(crossterm::cursor::MoveTo(h2_x, div_y + 2)).unwrap();
                print!("{}", help_l2);
                out.queue(ResetColor).unwrap();
            }
        }

        out.flush().unwrap();

        if let Ok(Event::Key(key)) = event::read() {
            if confirm_delete && !conns.is_empty() {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                        let active_conn = &conns[selected_idx];
                        let name = active_conn.nickname.clone();
                        match crate::storage::delete_connection(&name) {
                            Ok(_) => {
                                status_msg = Some(format!("✔ Connection '{}' deleted.", name));
                            }
                            Err(e) => {
                                status_msg = Some(format!("Error: {}", e));
                            }
                        }
                        confirm_delete = false;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        confirm_delete = false;
                    }
                    _ => {}
                }
                continue;
            }

            status_msg = None;

            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    break Ok(None);
                }
                KeyCode::Up => {
                    if !conns.is_empty() {
                        selected_idx = if selected_idx == 0 { conns.len() - 1 } else { selected_idx - 1 };
                    }
                }
                KeyCode::Down => {
                    if !conns.is_empty() {
                        selected_idx = (selected_idx + 1) % conns.len();
                    }
                }
                KeyCode::Enter => {
                    if !conns.is_empty() {
                        break Ok(Some(conns[selected_idx].clone()));
                    }
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    drop(_guard);
                    match run_add_wizard() {
                        Ok(Some(new_conn)) => {
                            let name = new_conn.nickname.clone();
                            if let Err(e) = crate::storage::add_connection(new_conn) {
                                status_msg = Some(format!("Save failed: {}", e));
                            } else {
                                status_msg = Some(format!("✔ Server '{}' saved!", name));
                            }
                        }
                        _ => {}
                    }
                    _guard = TerminalGuard::create()?;
                }
                KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Delete => {
                    if !conns.is_empty() {
                        confirm_delete = true;
                    }
                }
                _ => {}
            }
        }
    };

    result
}
