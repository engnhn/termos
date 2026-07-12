use crate::storage::ServerConnection;
use super::common::{draw_box, check_size_or_draw_error, TerminalGuard};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use std::io::{stdout, Write};

pub struct FormField {
    pub label: &'static str,
    pub value: String,
    pub is_password: bool,
    pub placeholder: &'static str,
    pub cursor_pos: usize,
    pub scroll_offset: usize,
}

pub fn run_wizard(existing: Option<&ServerConnection>) -> Result<Option<ServerConnection>, String> {
    let nick_val = existing.map(|e| e.nickname.clone()).unwrap_or_default();
    let nick_len = nick_val.chars().count();
    let host_val = existing.map(|e| e.host.clone()).unwrap_or_default();
    let host_len = host_val.chars().count();
    let port_val = existing.map(|e| e.port.to_string()).unwrap_or_else(|| "22".to_string());
    let port_len = port_val.chars().count();
    let user_val = existing.map(|e| e.username.clone()).unwrap_or_else(|| "root".to_string());
    let user_len = user_val.chars().count();
    let pass_val = existing.and_then(|e| e.password.clone()).unwrap_or_default();
    let pass_len = pass_val.chars().count();
    let key_val = existing.and_then(|e| e.ssh_key.clone()).unwrap_or_default();
    let key_len = key_val.chars().count();
    let grp_val = existing.and_then(|e| e.group.clone()).unwrap_or_default();
    let grp_len = grp_val.chars().count();

    let mut fields = vec![
        FormField {
            label: "Nickname",
            value: nick_val,
            is_password: false,
            placeholder: "e.g. my-server",
            cursor_pos: nick_len,
            scroll_offset: 0,
        },
        FormField {
            label: "IP / Hostname",
            value: host_val,
            is_password: false,
            placeholder: "e.g. 192.168.1.100",
            cursor_pos: host_len,
            scroll_offset: 0,
        },
        FormField {
            label: "Port",
            value: port_val,
            is_password: false,
            placeholder: "22",
            cursor_pos: port_len,
            scroll_offset: 0,
        },
        FormField {
            label: "Username",
            value: user_val,
            is_password: false,
            placeholder: "root",
            cursor_pos: user_len,
            scroll_offset: 0,
        },
        FormField {
            label: "Password (Optional)",
            value: pass_val,
            is_password: true,
            placeholder: "••••••••",
            cursor_pos: pass_len,
            scroll_offset: 0,
        },
        FormField {
            label: "SSH Key Path (Optional)",
            value: key_val,
            is_password: false,
            placeholder: "e.g. /home/user/.ssh/id_rsa",
            cursor_pos: key_len,
            scroll_offset: 0,
        },
        FormField {
            label: "Group (Optional)",
            value: grp_val,
            is_password: false,
            placeholder: "e.g. production",
            cursor_pos: grp_len,
            scroll_offset: 0,
        },
    ];

    let _guard = TerminalGuard::create()?;
    let mut out = stdout();

    let mut active_idx = 0;
    let mut error_msg: Option<String> = None;
    let box_width = 66;
    let box_height = 20;

    let result = loop {
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

        let title = if existing.is_some() { " TERMOS - EDIT SERVER " } else { " TERMOS - ADD SERVER " };
        draw_box(&mut out, start_x, start_y, box_width, box_height, title);

        for (i, field) in fields.iter_mut().enumerate() {
            let is_focused = active_idx == i;
            let field_y = start_y + 2 + i as u16 * 2;

            out.queue(crossterm::cursor::MoveTo(start_x + 3, field_y)).unwrap();
            if is_focused {
                out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                out.queue(SetAttribute(Attribute::Bold)).unwrap();
                print!("▶ ");
            } else {
                print!("  ");
            }
            print!("{:<24} : ", field.label);
            out.queue(ResetColor).unwrap();
            out.queue(SetAttribute(Attribute::Reset)).unwrap();

            let display_width = 27;

            if field.value.is_empty() {
                if is_focused {
                    out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                    print!("[");
                    out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                    let placeholder_text = format!(" {}", field.placeholder);
                    let truncated_placeholder: String = placeholder_text.chars().take(display_width).collect();
                    print!("{:<width$}", truncated_placeholder, width = display_width);
                    out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                    print!("]");
                } else {
                    out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                    let placeholder_text = format!(" {}", field.placeholder);
                    let truncated_placeholder: String = placeholder_text.chars().take(display_width).collect();
                    print!(" [{:<width$}]", truncated_placeholder, width = display_width);
                }
            } else {
                let val_len = field.value.chars().count();
                let c_pos = field.cursor_pos.min(val_len);

                let mut scroll_offset = field.scroll_offset;
                if c_pos < scroll_offset {
                    scroll_offset = c_pos;
                } else if c_pos > scroll_offset + display_width - 1 {
                    scroll_offset = c_pos - (display_width - 1);
                }
                field.scroll_offset = scroll_offset;

                let char_vec: Vec<char> = if field.is_password {
                    std::iter::repeat('•').take(val_len).collect()
                } else {
                    field.value.chars().collect()
                };

                let char_take = display_width - 1;
                let visible_chars: String = char_vec
                    .iter()
                    .skip(scroll_offset)
                    .take(char_take)
                    .collect();

                let val_to_show = format!(" {}", visible_chars);

                if is_focused {
                    out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                    print!("[");
                    out.queue(SetForegroundColor(Color::White)).unwrap();
                    print!("{:<width$}", val_to_show, width = display_width);
                    out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                    print!("]");
                } else {
                    out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                    print!(" ");
                    print!("{:<width$}", val_to_show, width = display_width);
                    print!(" ");
                }
            }
            out.queue(ResetColor).unwrap();
        }

        if active_idx < 7 {
            let field = &fields[active_idx];
            let val_len = field.value.chars().count();
            let c_pos = field.cursor_pos.min(val_len);
            let cursor_col = start_x + 34 + (c_pos - field.scroll_offset) as u16;
            let cursor_row = start_y + 2 + active_idx as u16 * 2;
            out.queue(crossterm::cursor::MoveTo(cursor_col, cursor_row)).unwrap();
            out.queue(crossterm::cursor::Show).unwrap();
        } else {
            out.queue(crossterm::cursor::Hide).unwrap();
        }

        let div_y = start_y + box_height - 5;
        out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
        let help_lines = [
            "Navigation: [Up/Down/Tab/Shift-Tab] to move between fields",
            "Shortcuts:  [ESC] to Cancel  |  [Ctrl+S] or button to Save",
        ];
        for (i, line) in help_lines.iter().enumerate() {
            let help_y = div_y + 1 + i as u16;
            let line_len = line.chars().count() as u16;
            let help_x = start_x + (box_width - line_len) / 2;
            out.queue(crossterm::cursor::MoveTo(help_x, help_y)).unwrap();
            print!("{}", line);
        }
        out.queue(ResetColor).unwrap();

        let btn_y = start_y + box_height - 4;
        
        out.queue(crossterm::cursor::MoveTo(start_x + 12, btn_y)).unwrap();
        if active_idx == 7 {
            out.queue(SetForegroundColor(Color::Black)).unwrap();
            out.queue(crossterm::style::SetBackgroundColor(Color::Green)).unwrap();
            out.queue(SetAttribute(Attribute::Bold)).unwrap();
            print!("    SAVE (Enter)    ");
        } else {
            out.queue(SetForegroundColor(Color::Green)).unwrap();
            out.queue(SetAttribute(Attribute::Bold)).unwrap();
            print!(" [ SAVE ] ");
        }
        out.queue(ResetColor).unwrap();
        out.queue(crossterm::style::SetBackgroundColor(Color::Reset)).unwrap();
        out.queue(SetAttribute(Attribute::Reset)).unwrap();

        out.queue(crossterm::cursor::MoveTo(start_x + box_width - 24, btn_y)).unwrap();
        if active_idx == 8 {
            out.queue(SetForegroundColor(Color::Black)).unwrap();
            out.queue(crossterm::style::SetBackgroundColor(Color::Red)).unwrap();
            out.queue(SetAttribute(Attribute::Bold)).unwrap();
            print!("   CANCEL (Esc)   ");
        } else {
            out.queue(SetForegroundColor(Color::Red)).unwrap();
            out.queue(SetAttribute(Attribute::Bold)).unwrap();
            print!(" [ CANCEL ] ");
        }
        out.queue(ResetColor).unwrap();
        out.queue(crossterm::style::SetBackgroundColor(Color::Reset)).unwrap();
        out.queue(SetAttribute(Attribute::Reset)).unwrap();

        if let Some(ref err) = error_msg {
            let err_y = start_y + box_height - 3;
            let clean_err = format!(" Error: {} ", err);
            let err_x = start_x + (box_width.saturating_sub(clean_err.chars().count() as u16)) / 2;
            out.queue(crossterm::cursor::MoveTo(err_x, err_y)).unwrap();
            out.queue(SetForegroundColor(Color::Black)).unwrap();
            out.queue(crossterm::style::SetBackgroundColor(Color::Red)).unwrap();
            out.queue(SetAttribute(Attribute::Bold)).unwrap();
            print!("{}", clean_err);
            out.queue(ResetColor).unwrap();
            out.queue(crossterm::style::SetBackgroundColor(Color::Reset)).unwrap();
            out.queue(SetAttribute(Attribute::Reset)).unwrap();
        }

        out.flush().unwrap();

        if let Ok(Event::Key(key)) = event::read() {
            if key.code == KeyCode::Esc {
                break Ok(None);
            }
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
                match validate_and_build(&fields, existing) {
                    Ok(conn) => break Ok(Some(conn)),
                    Err(e) => {
                        error_msg = Some(e);
                        continue;
                    }
                }
            }

            match key.code {
                KeyCode::Up => {
                    active_idx = if active_idx == 0 { 8 } else { active_idx - 1 };
                    error_msg = None;
                }
                KeyCode::Down | KeyCode::Tab => {
                    active_idx = if active_idx == 8 { 0 } else { active_idx + 1 };
                    error_msg = None;
                }
                KeyCode::BackTab => {
                    active_idx = if active_idx == 0 { 8 } else { active_idx - 1 };
                    error_msg = None;
                }
                KeyCode::Left => {
                    if active_idx < 7 {
                        let field = &mut fields[active_idx];
                        if field.cursor_pos > 0 {
                            field.cursor_pos -= 1;
                        }
                    } else if active_idx == 8 {
                        active_idx = 7;
                    } else if active_idx == 7 {
                        active_idx = 8;
                    }
                }
                KeyCode::Right => {
                    if active_idx < 7 {
                        let field = &mut fields[active_idx];
                        let val_len = field.value.chars().count();
                        if field.cursor_pos < val_len {
                            field.cursor_pos += 1;
                        }
                    } else if active_idx == 7 {
                        active_idx = 8;
                    } else if active_idx == 8 {
                        active_idx = 7;
                    }
                }
                KeyCode::Home => {
                    if active_idx < 7 {
                        fields[active_idx].cursor_pos = 0;
                    }
                }
                KeyCode::End => {
                    if active_idx < 7 {
                        fields[active_idx].cursor_pos = fields[active_idx].value.chars().count();
                    }
                }
                KeyCode::Delete => {
                    if active_idx < 7 {
                        let field = &mut fields[active_idx];
                        let char_count = field.value.chars().count();
                        if field.cursor_pos < char_count {
                            let mut chars: Vec<char> = field.value.chars().collect();
                            chars.remove(field.cursor_pos);
                            field.value = chars.into_iter().collect();
                        }
                        error_msg = None;
                    }
                }
                KeyCode::Enter => {
                    if active_idx < 7 {
                        active_idx += 1;
                    } else if active_idx == 7 {
                        match validate_and_build(&fields, existing) {
                            Ok(conn) => break Ok(Some(conn)),
                            Err(e) => {
                                error_msg = Some(e);
                            }
                        }
                    } else if active_idx == 8 {
                        break Ok(None);
                    }
                }
                KeyCode::Backspace => {
                    if active_idx < 7 {
                        let field = &mut fields[active_idx];
                        let char_count = field.value.chars().count();
                        if field.cursor_pos > 0 && field.cursor_pos <= char_count {
                            let mut chars: Vec<char> = field.value.chars().collect();
                            chars.remove(field.cursor_pos - 1);
                            field.value = chars.into_iter().collect();
                            field.cursor_pos -= 1;
                        }
                        error_msg = None;
                    }
                }
                KeyCode::Char(c) => {
                    if active_idx < 7 {
                        let field = &mut fields[active_idx];
                        let char_count = field.value.chars().count();
                        if field.cursor_pos <= char_count {
                            let mut chars: Vec<char> = field.value.chars().collect();
                            chars.insert(field.cursor_pos, c);
                            field.value = chars.into_iter().collect();
                            field.cursor_pos += 1;
                        }
                        error_msg = None;
                    }
                }
                _ => {}
            }
        }
    };

    result
}

fn validate_and_build(fields: &[FormField], existing: Option<&ServerConnection>) -> Result<ServerConnection, String> {
    let nickname = fields[0].value.trim().to_string();
    let host = fields[1].value.trim().to_string();
    let port_str = fields[2].value.trim();
    let username = fields[3].value.trim().to_string();
    let password = fields[4].value.trim().to_string();
    let ssh_key = fields[5].value.trim().to_string();
    let group_str = fields[6].value.trim().to_string();

    if nickname.is_empty() {
        return Err("Nickname cannot be empty.".to_string());
    }
    if host.is_empty() {
        return Err("IP / Hostname cannot be empty.".to_string());
    }
    if username.is_empty() {
        return Err("Username cannot be empty.".to_string());
    }

    let port: u16 = port_str
        .parse()
        .map_err(|_| "Invalid port number (1-65535).".to_string())?;

    let opt_password = if password.is_empty() { None } else { Some(password) };
    let opt_ssh_key = if ssh_key.is_empty() { None } else { Some(ssh_key) };
    let opt_group = if group_str.is_empty() { None } else { Some(group_str) };

    let quick_commands = existing.and_then(|e| e.quick_commands.clone());

    Ok(ServerConnection {
        nickname,
        host,
        port,
        username,
        password: opt_password,
        ssh_key: opt_ssh_key,
        group: opt_group,
        quick_commands,
    })
}

pub fn run_qc_wizard(existing: Option<&crate::storage::QuickCommand>) -> Result<Option<crate::storage::QuickCommand>, String> {
    let name_val = existing.map(|e| e.name.clone()).unwrap_or_default();
    let name_len = name_val.chars().count();
    let cmd_val = existing.map(|e| e.command.clone()).unwrap_or_default();
    let cmd_len = cmd_val.chars().count();

    let mut fields = vec![
        FormField {
            label: "Command Name",
            value: name_val,
            is_password: false,
            placeholder: "e.g. Disk Usage",
            cursor_pos: name_len,
            scroll_offset: 0,
        },
        FormField {
            label: "SSH Command",
            value: cmd_val,
            is_password: false,
            placeholder: "e.g. df -h",
            cursor_pos: cmd_len,
            scroll_offset: 0,
        },
    ];

    let _guard = TerminalGuard::create()?;
    let mut out = stdout();

    let mut active_idx = 0;
    let mut error_msg: Option<String> = None;
    let box_width = 54;
    let box_height = 10;

    let result = loop {
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

        let title = if existing.is_some() { " EDIT QUICK COMMAND " } else { " ADD QUICK COMMAND " };
        draw_box(&mut out, start_x, start_y, box_width, box_height, title);

        for (i, field) in fields.iter_mut().enumerate() {
            let is_focused = active_idx == i;
            let field_y = start_y + 2 + i as u16 * 2;

            out.queue(crossterm::cursor::MoveTo(start_x + 3, field_y)).unwrap();
            if is_focused {
                out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                out.queue(SetAttribute(Attribute::Bold)).unwrap();
                print!("▶ ");
            } else {
                print!("  ");
            }
            print!("{:<14} : ", field.label);
            out.queue(ResetColor).unwrap();
            out.queue(SetAttribute(Attribute::Reset)).unwrap();

            let display_width = 26;

            if field.value.is_empty() {
                if is_focused {
                    out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                    print!("[");
                    out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                    let placeholder_text = format!(" {}", field.placeholder);
                    let truncated_placeholder: String = placeholder_text.chars().take(display_width).collect();
                    print!("{:<width$}", truncated_placeholder, width = display_width);
                    out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                    print!("]");
                } else {
                    out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                    let placeholder_text = format!(" {}", field.placeholder);
                    let truncated_placeholder: String = placeholder_text.chars().take(display_width).collect();
                    print!(" [{:<width$}]", truncated_placeholder, width = display_width);
                }
            } else {
                let val_len = field.value.chars().count();
                let c_pos = field.cursor_pos.min(val_len);

                let mut scroll_offset = field.scroll_offset;
                if c_pos < scroll_offset {
                    scroll_offset = c_pos;
                } else if c_pos > scroll_offset + display_width - 1 {
                    scroll_offset = c_pos - (display_width - 1);
                }
                field.scroll_offset = scroll_offset;

                let char_vec: Vec<char> = field.value.chars().collect();
                let char_take = display_width - 1;
                let visible_chars: String = char_vec
                    .iter()
                    .skip(scroll_offset)
                    .take(char_take)
                    .collect();

                let val_to_show = format!(" {}", visible_chars);

                if is_focused {
                    out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                    print!("[");
                    out.queue(SetForegroundColor(Color::White)).unwrap();
                    print!("{:<width$}", val_to_show, width = display_width);
                    out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                    print!("]");
                } else {
                    out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                    print!(" ");
                    print!("{:<width$}", val_to_show, width = display_width);
                    print!(" ");
                }
            }
            out.queue(ResetColor).unwrap();
        }

        if active_idx < 2 {
            let field = &fields[active_idx];
            let val_len = field.value.chars().count();
            let c_pos = field.cursor_pos.min(val_len);
            let cursor_col = start_x + 24 + (c_pos - field.scroll_offset) as u16;
            let cursor_row = start_y + 2 + active_idx as u16 * 2;
            out.queue(crossterm::cursor::MoveTo(cursor_col, cursor_row)).unwrap();
            out.queue(crossterm::cursor::Show).unwrap();
        } else {
            out.queue(crossterm::cursor::Hide).unwrap();
        }

        let btn_y = start_y + box_height - 3;
        
        out.queue(crossterm::cursor::MoveTo(start_x + 8, btn_y)).unwrap();
        if active_idx == 2 {
            out.queue(SetForegroundColor(Color::Black)).unwrap();
            out.queue(crossterm::style::SetBackgroundColor(Color::Green)).unwrap();
            out.queue(SetAttribute(Attribute::Bold)).unwrap();
            print!("   SAVE   ");
        } else {
            out.queue(SetForegroundColor(Color::Green)).unwrap();
            out.queue(SetAttribute(Attribute::Bold)).unwrap();
            print!(" [ SAVE ] ");
        }
        out.queue(ResetColor).unwrap();
        out.queue(crossterm::style::SetBackgroundColor(Color::Reset)).unwrap();
        out.queue(SetAttribute(Attribute::Reset)).unwrap();

        out.queue(crossterm::cursor::MoveTo(start_x + box_width - 18, btn_y)).unwrap();
        if active_idx == 3 {
            out.queue(SetForegroundColor(Color::Black)).unwrap();
            out.queue(crossterm::style::SetBackgroundColor(Color::Red)).unwrap();
            out.queue(SetAttribute(Attribute::Bold)).unwrap();
            print!("  CANCEL  ");
        } else {
            out.queue(SetForegroundColor(Color::Red)).unwrap();
            out.queue(SetAttribute(Attribute::Bold)).unwrap();
            print!(" [ CANCEL ] ");
        }
        out.queue(ResetColor).unwrap();
        out.queue(crossterm::style::SetBackgroundColor(Color::Reset)).unwrap();
        out.queue(SetAttribute(Attribute::Reset)).unwrap();

        if let Some(ref err) = error_msg {
            let err_y = start_y + box_height - 2;
            let clean_err = format!(" Error: {} ", err);
            let err_x = start_x + (box_width.saturating_sub(clean_err.chars().count() as u16)) / 2;
            out.queue(crossterm::cursor::MoveTo(err_x, err_y)).unwrap();
            out.queue(SetForegroundColor(Color::Black)).unwrap();
            out.queue(crossterm::style::SetBackgroundColor(Color::Red)).unwrap();
            out.queue(SetAttribute(Attribute::Bold)).unwrap();
            print!("{}", clean_err);
            out.queue(ResetColor).unwrap();
            out.queue(crossterm::style::SetBackgroundColor(Color::Reset)).unwrap();
            out.queue(SetAttribute(Attribute::Reset)).unwrap();
        }

        out.flush().unwrap();

        if let Ok(Event::Key(key)) = event::read() {
            if key.code == KeyCode::Esc {
                break Ok(None);
            }

            match key.code {
                KeyCode::Up => {
                    active_idx = if active_idx == 0 { 3 } else { active_idx - 1 };
                    error_msg = None;
                }
                KeyCode::Down | KeyCode::Tab => {
                    active_idx = if active_idx == 3 { 0 } else { active_idx + 1 };
                    error_msg = None;
                }
                KeyCode::BackTab => {
                    active_idx = if active_idx == 0 { 3 } else { active_idx - 1 };
                    error_msg = None;
                }
                KeyCode::Left => {
                    if active_idx < 2 {
                        let field = &mut fields[active_idx];
                        if field.cursor_pos > 0 {
                            field.cursor_pos -= 1;
                        }
                    } else if active_idx == 3 {
                        active_idx = 2;
                    } else if active_idx == 2 {
                        active_idx = 3;
                    }
                }
                KeyCode::Right => {
                    if active_idx < 2 {
                        let field = &mut fields[active_idx];
                        let val_len = field.value.chars().count();
                        if field.cursor_pos < val_len {
                            field.cursor_pos += 1;
                        }
                    } else if active_idx == 2 {
                        active_idx = 3;
                    } else if active_idx == 3 {
                        active_idx = 2;
                    }
                }
                KeyCode::Home => {
                    if active_idx < 2 {
                        fields[active_idx].cursor_pos = 0;
                    }
                }
                KeyCode::End => {
                    if active_idx < 2 {
                        fields[active_idx].cursor_pos = fields[active_idx].value.chars().count();
                    }
                }
                KeyCode::Delete => {
                    if active_idx < 2 {
                        let field = &mut fields[active_idx];
                        let char_count = field.value.chars().count();
                        if field.cursor_pos < char_count {
                            let mut chars: Vec<char> = field.value.chars().collect();
                            chars.remove(field.cursor_pos);
                            field.value = chars.into_iter().collect();
                        }
                        error_msg = None;
                    }
                }
                KeyCode::Enter => {
                    if active_idx < 2 {
                        active_idx += 1;
                    } else if active_idx == 2 {
                        let name = fields[0].value.trim().to_string();
                        let command = fields[1].value.trim().to_string();
                        if name.is_empty() || command.is_empty() {
                            error_msg = Some("Fields cannot be empty.".to_string());
                        } else {
                            break Ok(Some(crate::storage::QuickCommand { name, command }));
                        }
                    } else if active_idx == 3 {
                        break Ok(None);
                    }
                }
                KeyCode::Backspace => {
                    if active_idx < 2 {
                        let field = &mut fields[active_idx];
                        let char_count = field.value.chars().count();
                        if field.cursor_pos > 0 && field.cursor_pos <= char_count {
                            let mut chars: Vec<char> = field.value.chars().collect();
                            chars.remove(field.cursor_pos - 1);
                            field.value = chars.into_iter().collect();
                            field.cursor_pos -= 1;
                        }
                        error_msg = None;
                    }
                }
                KeyCode::Char(c) => {
                    if active_idx < 2 {
                        let field = &mut fields[active_idx];
                        let char_count = field.value.chars().count();
                        if field.cursor_pos <= char_count {
                            let mut chars: Vec<char> = field.value.chars().collect();
                            chars.insert(field.cursor_pos, c);
                            field.value = chars.into_iter().collect();
                            field.cursor_pos += 1;
                        }
                        error_msg = None;
                    }
                }
                _ => {}
            }
        }
    };

    result
}
