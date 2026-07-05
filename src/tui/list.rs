use crate::storage::ServerConnection;
use super::common::{draw_box, check_size_or_draw_error, TerminalGuard};
use super::wizard::run_wizard;
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
    let mut active_group: Option<String> = None;
    
    let mut show_group_select = false;
    let mut group_select_idx = 0;
    let mut groups: Vec<String> = Vec::new();

    let mut show_quick_commands = false;
    let mut quick_command_idx = 0;

    let mut show_manage_server = false;
    let mut manage_server_idx = 0;

    let box_width = 76;
    let box_height = 18;

    let result = loop {
        let all_conns = match crate::storage::load_connections() {
            Ok(list) => list,
            Err(e) => {
                break Err(format!("Database error: {}", e));
            }
        };

        let conns: Vec<ServerConnection> = if let Some(ref grp) = active_group {
            all_conns.clone().into_iter().filter(|c| c.group.as_ref() == Some(grp)).collect()
        } else {
            all_conns.clone()
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

        let title = if let Some(ref grp) = active_group {
            format!(" TERMOS - CONNECTIONS ({}) ", grp)
        } else {
            " TERMOS - CONNECTIONS ".to_string()
        };

        draw_box(&mut out, start_x, start_y, box_width, box_height, &title);

        if conns.is_empty() {
            let empty_msg = if active_group.is_some() {
                "No servers in this group."
            } else {
                "No servers registered yet."
            };
            let x = start_x + box_width.saturating_sub(empty_msg.chars().count() as u16) / 2;
            out.queue(crossterm::cursor::MoveTo(x, start_y + 5)).unwrap();
            out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
            print!("{}", empty_msg);

            let add_hint = "Press [a] to add a new server or [g] to filter.";
            let x2 = start_x + box_width.saturating_sub(add_hint.chars().count() as u16) / 2;
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
                let group_tag = if let Some(ref g) = conn.group {
                    format!(" [{}]", g)
                } else {
                    "".to_string()
                };

                let display_str = format!("{:<14}{} ({}){}", conn.nickname, group_tag, format!("{}@{}:{}", conn.username, conn.host, conn.port), key_tag);
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
                out.queue(crossterm::cursor::MoveTo(start_x + box_width.saturating_sub(4), start_y + 2)).unwrap();
                out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                print!("▲");
            }
            if scroll_offset + 8 < conns.len() {
                out.queue(crossterm::cursor::MoveTo(start_x + box_width.saturating_sub(4), start_y + 9)).unwrap();
                out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                print!("▼");
            }
            out.queue(ResetColor).unwrap();
        }

        let div_y = start_y + box_height - 5;
        
        if confirm_delete && !conns.is_empty() {
            let active_conn = &conns[selected_idx];
            let confirm_line = format!("Delete '{}'? (y/n)", active_conn.nickname);
            let confirm_x = start_x + box_width.saturating_sub(confirm_line.chars().count() as u16) / 2;
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
                let status_x = start_x + box_width.saturating_sub(status.chars().count() as u16) / 2;
                out.queue(crossterm::cursor::MoveTo(status_x, div_y + 1)).unwrap();
                out.queue(SetForegroundColor(Color::Green)).unwrap();
                out.queue(SetAttribute(Attribute::Bold)).unwrap();
                print!("{}", status);
                out.queue(ResetColor).unwrap();
                out.queue(SetAttribute(Attribute::Reset)).unwrap();
            } else {
                let help_l1 = "Navigate: [Up/Down] arrows  |  Connect: [Enter]";
                let help_l2 = "Actions:  [a] Add  |  [d] Del  |  [e] Manage  |  [g] Group  |  [c] Cmd";

                let h1_x = start_x + box_width.saturating_sub(help_l1.chars().count() as u16) / 2;
                let h2_x = start_x + box_width.saturating_sub(help_l2.chars().count() as u16) / 2;

                out.queue(crossterm::cursor::MoveTo(h1_x, div_y + 1)).unwrap();
                out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                print!("{}", help_l1);

                out.queue(crossterm::cursor::MoveTo(h2_x, div_y + 2)).unwrap();
                print!("{}", help_l2);
                out.queue(ResetColor).unwrap();
            }
        }

        if show_group_select {
            let overlay_w = 40;
            let overlay_h = (groups.len() + 4).max(6) as u16;
            let ox = start_x + box_width.saturating_sub(overlay_w) / 2;
            let oy = start_y + box_height.saturating_sub(overlay_h) / 2;
            draw_box(&mut out, ox, oy, overlay_w, overlay_h, " SELECT GROUP ");
            
            out.queue(crossterm::cursor::MoveTo(ox + 3, oy + 2)).unwrap();
            if group_select_idx == 0 {
                out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                out.queue(SetAttribute(Attribute::Bold)).unwrap();
                print!("▶ ");
                out.queue(SetForegroundColor(Color::White)).unwrap();
                print!("[ Show All ]");
            } else {
                print!("  [ Show All ]");
            }
            out.queue(ResetColor).unwrap();
            out.queue(SetAttribute(Attribute::Reset)).unwrap();

            for (gi, gname) in groups.iter().enumerate() {
                let row_y = oy + 3 + gi as u16;
                out.queue(crossterm::cursor::MoveTo(ox + 3, row_y)).unwrap();
                let is_focused = group_select_idx == gi + 1;
                if is_focused {
                    out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                    out.queue(SetAttribute(Attribute::Bold)).unwrap();
                    print!("▶ ");
                    out.queue(SetForegroundColor(Color::White)).unwrap();
                    print!("{}", gname);
                } else {
                    print!("  ");
                    out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                    print!("{}", gname);
                }
                out.queue(ResetColor).unwrap();
                out.queue(SetAttribute(Attribute::Reset)).unwrap();
            }
        }

        if show_manage_server {
            if let Some(active_conn) = conns.get(selected_idx) {
                let overlay_w = 40;
                let overlay_h = 8;
                let ox = start_x + box_width.saturating_sub(overlay_w) / 2;
                let oy = start_y + box_height.saturating_sub(overlay_h) / 2;
                
                let title = format!(" MANAGE: {} ", active_conn.nickname);
                draw_box(&mut out, ox, oy, overlay_w, overlay_h, &title);

                let options = ["1. Edit Server Fields", "2. Manage Quick Commands"];
                for (oi, opt) in options.iter().enumerate() {
                    let row_y = oy + 2 + oi as u16 * 2;
                    out.queue(crossterm::cursor::MoveTo(ox + 4, row_y)).unwrap();
                    let is_focused = manage_server_idx == oi;
                    if is_focused {
                        out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                        out.queue(SetAttribute(Attribute::Bold)).unwrap();
                        print!("▶ ");
                        out.queue(SetForegroundColor(Color::White)).unwrap();
                        print!("{}", opt);
                    } else {
                        print!("  ");
                        out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                        print!("{}", opt);
                    }
                    out.queue(ResetColor).unwrap();
                    out.queue(SetAttribute(Attribute::Reset)).unwrap();
                }
            }
        }

        if show_quick_commands {
            if let Some(active_conn) = conns.get(selected_idx) {
                if let Some(ref cmds) = active_conn.quick_commands {
                    let overlay_w = 46;
                    let overlay_h = (cmds.len() + 4).max(6) as u16;
                    let ox = start_x + box_width.saturating_sub(overlay_w) / 2;
                    let oy = start_y + box_height.saturating_sub(overlay_h) / 2;
                    draw_box(&mut out, ox, oy, overlay_w, overlay_h, " QUICK COMMANDS ");

                    for (ci, cmd) in cmds.iter().enumerate() {
                        let row_y = oy + 2 + ci as u16;
                        out.queue(crossterm::cursor::MoveTo(ox + 3, row_y)).unwrap();
                        let is_focused = quick_command_idx == ci;
                        
                        let display_cmd = format!("{}: {}", cmd.name, cmd.command);
                        let truncated_cmd: String = display_cmd.chars().take((overlay_w - 8) as usize).collect();

                        if is_focused {
                            out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                            out.queue(SetAttribute(Attribute::Bold)).unwrap();
                            print!("▶ ");
                            out.queue(SetForegroundColor(Color::White)).unwrap();
                            print!("{:<width$}", truncated_cmd, width = (overlay_w - 8) as usize);
                        } else {
                            print!("  ");
                            out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                            print!("{:<width$}", truncated_cmd, width = (overlay_w - 8) as usize);
                        }
                        out.queue(ResetColor).unwrap();
                        out.queue(SetAttribute(Attribute::Reset)).unwrap();
                    }
                }
            }
        }

        out.flush().unwrap();

        if let Ok(Event::Key(key)) = event::read() {
            if show_group_select {
                match key.code {
                    KeyCode::Esc => {
                        show_group_select = false;
                    }
                    KeyCode::Up => {
                        let total_options = groups.len() + 1;
                        group_select_idx = if group_select_idx == 0 { total_options - 1 } else { group_select_idx - 1 };
                    }
                    KeyCode::Down => {
                        let total_options = groups.len() + 1;
                        group_select_idx = (group_select_idx + 1) % total_options;
                    }
                    KeyCode::Enter => {
                        if group_select_idx == 0 {
                            active_group = None;
                        } else {
                            active_group = Some(groups[group_select_idx - 1].clone());
                        }
                        selected_idx = 0;
                        scroll_offset = 0;
                        show_group_select = false;
                    }
                    _ => {}
                }
                continue;
            }

            if show_quick_commands {
                if let Some(active_conn) = conns.get(selected_idx) {
                    if let Some(ref cmds) = active_conn.quick_commands {
                        match key.code {
                            KeyCode::Esc => {
                                show_quick_commands = false;
                            }
                            KeyCode::Up => {
                                quick_command_idx = if quick_command_idx == 0 { cmds.len() - 1 } else { quick_command_idx - 1 };
                            }
                            KeyCode::Down => {
                                quick_command_idx = (quick_command_idx + 1) % cmds.len();
                            }
                            KeyCode::Enter => {
                                let selected_cmd = &cmds[quick_command_idx];
                                drop(_guard);
                                
                                out.queue(Clear(ClearType::All)).unwrap();
                                out.flush().unwrap();
                                
                                println!("\x1b[1;36m⚡ Executing '{}' on {}...\x1b[0m", selected_cmd.name, active_conn.nickname);
                                println!("\x1b[1;30mCommand: {}\x1b[0m\n", selected_cmd.command);
                                
                                match crate::ssh::execute_ssh_command(active_conn, &selected_cmd.command) {
                                    Ok(_) => {
                                        println!("\n\x1b[1;32m✔ Command execution finished successfully.\x1b[0m");
                                    }
                                    Err(e) => {
                                        eprintln!("\n\x1b[1;31mError: {}\x1b[0m", e);
                                    }
                                }
                                
                                println!("\x1b[1;33mPress Enter to return to Termos...\x1b[0m");
                                let mut buffer = String::new();
                                let _ = std::io::stdin().read_line(&mut buffer);
                                
                                _guard = TerminalGuard::create()?;
                                show_quick_commands = false;
                            }
                            _ => {}
                        }
                    }
                }
                continue;
            }

            if show_manage_server {
                if let Some(active_conn) = conns.get(selected_idx) {
                    match key.code {
                        KeyCode::Esc => {
                            show_manage_server = false;
                        }
                        KeyCode::Up => {
                            manage_server_idx = if manage_server_idx == 0 { 1 } else { manage_server_idx - 1 };
                        }
                        KeyCode::Down => {
                            manage_server_idx = (manage_server_idx + 1) % 2;
                        }
                        KeyCode::Enter => {
                            if manage_server_idx == 0 {
                                drop(_guard);
                                match run_wizard(Some(active_conn)) {
                                    Ok(Some(updated)) => {
                                        if let Err(e) = crate::storage::update_connection(&active_conn.nickname, updated) {
                                            status_msg = Some(format!("Edit failed: {}", e));
                                        } else {
                                            status_msg = Some("✔ Server changes saved!".to_string());
                                        }
                                    }
                                    _ => {}
                                }
                                _guard = TerminalGuard::create()?;
                                show_manage_server = false;
                            } else {
                                show_manage_server = false;
                                drop(_guard);
                                let _ = super::qc_manager::run_qc_manager(Some(active_conn.nickname.clone()), super::qc_manager::QcMode::List);
                                _guard = TerminalGuard::create()?;
                            }
                        }
                        _ => {}
                    }
                }
                continue;
            }

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
                KeyCode::Char('g') | KeyCode::Char('G') | KeyCode::Char('/') => {
                    let mut unique_groups = Vec::new();
                    for c in &all_conns {
                        if let Some(ref g) = c.group {
                            if !unique_groups.contains(g) {
                                unique_groups.push(g.clone());
                            }
                        }
                    }
                    if unique_groups.is_empty() {
                        status_msg = Some("No groups defined yet.".to_string());
                    } else {
                        groups = unique_groups;
                        group_select_idx = 0;
                        show_group_select = true;
                    }
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    if !conns.is_empty() {
                        let active_conn = &conns[selected_idx];
                        if active_conn.quick_commands.is_some() {
                            quick_command_idx = 0;
                            show_quick_commands = true;
                        } else {
                            status_msg = Some("No quick commands for this server.".to_string());
                        }
                    }
                }
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    if !conns.is_empty() {
                        manage_server_idx = 0;
                        show_manage_server = true;
                    }
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    drop(_guard);
                    match run_wizard(None) {
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
