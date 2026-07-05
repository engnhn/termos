use crate::storage::load_connections;
use super::common::{draw_box, check_size_or_draw_error, TerminalGuard};
use super::wizard::run_qc_wizard;
use crossterm::{
    event::{self, Event, KeyCode},
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use std::io::{stdout, Write};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum QcMode {
    List,
    Add,
    Edit,
    Delete,
}

pub fn run_qc_manager(nickname: Option<String>, mode: QcMode) -> Result<(), String> {
    let mut _guard = TerminalGuard::create()?;
    let mut out = stdout();

    let is_fixed_server = nickname.is_some();
    let mut current_nickname = nickname;
    
    let mut server_selected_idx = 0;
    let mut server_scroll_offset = 0;
    
    let mut qc_selected_idx = 0;
    let mut qc_confirm_delete = false;
    let mut status_msg: Option<String> = None;

    let box_width = 60;
    let box_height = 14;

    loop {
        let all_conns = match load_connections() {
            Ok(list) => list,
            Err(e) => return Err(format!("Database error: {}", e)),
        };

        let current_conn = if let Some(ref name) = current_nickname {
            let found = all_conns.iter().find(|c| c.nickname.eq_ignore_ascii_case(name));
            if found.is_none() {
                return Err(format!("Server with nickname '{}' not found.", name));
            }
            found.cloned()
        } else {
            None
        };

        out.queue(Clear(ClearType::All)).unwrap();

        let (cols, rows) = match check_size_or_draw_error(&mut out, box_width, box_height) {
            Ok(Some(sz)) => sz,
            _ => {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.code == KeyCode::Esc {
                        break;
                    }
                }
                continue;
            }
        };

        let start_x = (cols.saturating_sub(box_width)) / 2;
        let start_y = (rows.saturating_sub(box_height)) / 2;

        if let Some(ref conn) = current_conn {
            let title = match mode {
                QcMode::List => format!(" QUICK COMMANDS: {} ", conn.nickname),
                QcMode::Delete => format!(" DELETE QC: {} ", conn.nickname),
                QcMode::Edit => format!(" EDIT QC: {} ", conn.nickname),
                QcMode::Add => format!(" ADD QC: {} ", conn.nickname),
            };

            draw_box(&mut out, start_x, start_y, box_width, box_height, &title);

            let qcs = conn.quick_commands.as_ref().cloned().unwrap_or_default();

            if mode == QcMode::Add {
                drop(_guard);
                match run_qc_wizard(None) {
                    Ok(Some(new_qc)) => {
                        if let Err(e) = crate::storage::add_quick_command(&conn.nickname, new_qc) {
                            status_msg = Some(format!("Add failed: {}", e));
                        } else {
                            status_msg = Some("✔ Command added.".to_string());
                        }
                    }
                    _ => {}
                }
                _guard = TerminalGuard::create()?;
                
                if is_fixed_server {
                    break;
                } else {
                    current_nickname = None;
                    continue;
                }
            }

            if qcs.is_empty() {
                let empty_msg = "No quick commands defined.";
                let x = start_x + (box_width - empty_msg.chars().count() as u16) / 2;
                out.queue(crossterm::cursor::MoveTo(x, start_y + 4)).unwrap();
                out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                print!("{}", empty_msg);
                out.queue(ResetColor).unwrap();
            } else {
                if qc_selected_idx >= qcs.len() {
                    qc_selected_idx = qcs.len() - 1;
                }

                for qi in 0..5 {
                    if qi >= qcs.len() {
                        break;
                    }
                    let qc = &qcs[qi];
                    let row_y = start_y + 2 + qi as u16;
                    out.queue(crossterm::cursor::MoveTo(start_x + 3, row_y)).unwrap();
                    let is_focused = qc_selected_idx == qi;

                    let display_qc = format!("{}: {}", qc.name, qc.command);
                    let truncated_qc: String = display_qc.chars().take((box_width - 8) as usize).collect();

                    if is_focused {
                        out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                        out.queue(SetAttribute(Attribute::Bold)).unwrap();
                        print!("▶ ");
                        out.queue(SetForegroundColor(Color::White)).unwrap();
                        print!("{:<width$}", truncated_qc, width = (box_width - 8) as usize);
                    } else {
                        print!("  ");
                        out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                        print!("{:<width$}", truncated_qc, width = (box_width - 8) as usize);
                    }
                    out.queue(ResetColor).unwrap();
                    out.queue(SetAttribute(Attribute::Reset)).unwrap();
                }
            }

            let div_y = start_y + box_height - 4;
            
            if qc_confirm_delete && !qcs.is_empty() {
                let target_qc = &qcs[qc_selected_idx];
                let confirm_line = format!("Delete '{}'? (y/n)", target_qc.name);
                let confirm_x = start_x + (box_width - confirm_line.chars().count() as u16) / 2;
                out.queue(crossterm::cursor::MoveTo(confirm_x, div_y + 1)).unwrap();
                out.queue(SetForegroundColor(Color::Black)).unwrap();
                out.queue(crossterm::style::SetBackgroundColor(Color::Red)).unwrap();
                out.queue(SetAttribute(Attribute::Bold)).unwrap();
                print!(" {} ", confirm_line);
                out.queue(ResetColor).unwrap();
                out.queue(crossterm::style::SetBackgroundColor(Color::Reset)).unwrap();
                out.queue(SetAttribute(Attribute::Reset)).unwrap();
            } else if let Some(ref status) = status_msg {
                let status_x = start_x + (box_width - status.chars().count() as u16) / 2;
                out.queue(crossterm::cursor::MoveTo(status_x, div_y + 1)).unwrap();
                out.queue(SetForegroundColor(Color::Green)).unwrap();
                out.queue(SetAttribute(Attribute::Bold)).unwrap();
                print!("{}", status);
                out.queue(ResetColor).unwrap();
                out.queue(SetAttribute(Attribute::Reset)).unwrap();
            } else {
                let help_l1 = match mode {
                    QcMode::List => "Navigate: [Up/Down] arrows  |  [Esc] Back",
                    QcMode::Delete => "Select command to Delete: [Enter]  |  [Esc] Back",
                    QcMode::Edit => "Select command to Edit: [Enter]  |  [Esc] Back",
                    _ => "",
                };
                let h1_x = start_x + (box_width - help_l1.chars().count() as u16) / 2;
                out.queue(crossterm::cursor::MoveTo(h1_x, div_y + 1)).unwrap();
                out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                print!("{}", help_l1);
                out.queue(ResetColor).unwrap();
            }

            out.flush().unwrap();

            if let Ok(Event::Key(key)) = event::read() {
                if qc_confirm_delete && !qcs.is_empty() {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                            let target_qc = &qcs[qc_selected_idx];
                            if let Err(e) = crate::storage::delete_quick_command(&conn.nickname, &target_qc.name) {
                                status_msg = Some(format!("Delete failed: {}", e));
                            } else {
                                status_msg = Some("✔ Command deleted.".to_string());
                            }
                            qc_confirm_delete = false;
                            qc_selected_idx = 0;
                        }
                        _ => {
                            qc_confirm_delete = false;
                        }
                    }
                    continue;
                }

                status_msg = None;

                match key.code {
                    KeyCode::Esc => {
                        if is_fixed_server {
                            break;
                        } else {
                            current_nickname = None;
                        }
                    }
                    KeyCode::Up => {
                        if !qcs.is_empty() {
                            qc_selected_idx = if qc_selected_idx == 0 { qcs.len() - 1 } else { qc_selected_idx - 1 };
                        }
                    }
                    KeyCode::Down => {
                        if !qcs.is_empty() {
                            qc_selected_idx = (qc_selected_idx + 1) % qcs.len();
                        }
                    }
                    KeyCode::Enter => {
                        if !qcs.is_empty() {
                            match mode {
                                QcMode::Delete => {
                                    qc_confirm_delete = true;
                                }
                                QcMode::Edit => {
                                    let target_qc = &qcs[qc_selected_idx];
                                    drop(_guard);
                                    match run_qc_wizard(Some(target_qc)) {
                                        Ok(Some(updated)) => {
                                            if let Err(e) = crate::storage::edit_quick_command(
                                                &conn.nickname,
                                                &target_qc.name,
                                                Some(updated.name),
                                                Some(updated.command)
                                            ) {
                                                status_msg = Some(format!("Edit failed: {}", e));
                                            } else {
                                                status_msg = Some("✔ Command updated.".to_string());
                                            }
                                        }
                                        _ => {}
                                    }
                                    _guard = TerminalGuard::create()?;
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        } else {
            draw_box(&mut out, start_x, start_y, box_width, box_height, " SELECT SERVER FOR QC ");

            if all_conns.is_empty() {
                let empty_msg = "No servers registered yet.";
                let x = start_x + (box_width - empty_msg.chars().count() as u16) / 2;
                out.queue(crossterm::cursor::MoveTo(x, start_y + 5)).unwrap();
                out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                print!("{}", empty_msg);
                out.queue(ResetColor).unwrap();
            } else {
                if server_selected_idx >= all_conns.len() {
                    server_selected_idx = all_conns.len() - 1;
                }

                if server_selected_idx < server_scroll_offset {
                    server_scroll_offset = server_selected_idx;
                } else if server_selected_idx >= server_scroll_offset + 5 {
                    server_scroll_offset = server_selected_idx - 4;
                }

                for i in 0..5 {
                    let idx = server_scroll_offset + i;
                    if idx >= all_conns.len() {
                        break;
                    }
                    let conn = &all_conns[idx];
                    let is_selected = idx == server_selected_idx;
                    let row_y = start_y + 2 + i as u16;

                    out.queue(crossterm::cursor::MoveTo(start_x + 3, row_y)).unwrap();

                    let group_tag = if let Some(ref g) = conn.group {
                        format!(" [{}]", g)
                    } else {
                        "".to_string()
                    };
                    let display_str = format!("{}{}", conn.nickname, group_tag);
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
            }

            let div_y = start_y + box_height - 4;
            let help_line = "Select server: [Up/Down]  |  Confirm: [Enter]  |  [Esc] Exit";
            let h_x = start_x + (box_width - help_line.chars().count() as u16) / 2;
            out.queue(crossterm::cursor::MoveTo(h_x, div_y + 1)).unwrap();
            out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
            print!("{}", help_line);
            out.queue(ResetColor).unwrap();

            out.flush().unwrap();

            if let Ok(Event::Key(key)) = event::read() {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Up => {
                        if !all_conns.is_empty() {
                            server_selected_idx = if server_selected_idx == 0 { all_conns.len() - 1 } else { server_selected_idx - 1 };
                        }
                    }
                    KeyCode::Down => {
                        if !all_conns.is_empty() {
                            server_selected_idx = (server_selected_idx + 1) % all_conns.len();
                        }
                    }
                    KeyCode::Enter => {
                        if !all_conns.is_empty() {
                            current_nickname = Some(all_conns[server_selected_idx].nickname.clone());
                            qc_selected_idx = 0;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
