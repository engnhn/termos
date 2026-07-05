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
}

pub fn run_add_wizard() -> Result<Option<ServerConnection>, String> {
    let mut fields = vec![
        FormField {
            label: "Nickname",
            value: String::new(),
            is_password: false,
            placeholder: "e.g. my-server",
        },
        FormField {
            label: "IP / Hostname",
            value: String::new(),
            is_password: false,
            placeholder: "e.g. 192.168.1.100",
        },
        FormField {
            label: "Port",
            value: "22".to_string(),
            is_password: false,
            placeholder: "22",
        },
        FormField {
            label: "Username",
            value: "root".to_string(),
            is_password: false,
            placeholder: "root",
        },
        FormField {
            label: "Password (Optional)",
            value: String::new(),
            is_password: true,
            placeholder: "••••••••",
        },
        FormField {
            label: "SSH Key Path (Optional)",
            value: String::new(),
            is_password: false,
            placeholder: "e.g. /home/user/.ssh/id_rsa",
        },
    ];

    let _guard = TerminalGuard::create()?;
    let mut out = stdout();

    let mut active_idx = 0;
    let mut error_msg: Option<String> = None;
    let box_width = 66;
    let box_height = 18;

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

        draw_box(&mut out, start_x, start_y, box_width, box_height, " TERMOS - ADD SERVER ");

        for (i, field) in fields.iter().enumerate() {
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

            let val_to_show = if field.value.is_empty() {
                out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                format!(" {}", field.placeholder)
            } else if field.is_password {
                out.queue(SetForegroundColor(Color::Yellow)).unwrap();
                format!(" {}", "•".repeat(field.value.len()))
            } else {
                out.queue(SetForegroundColor(Color::White)).unwrap();
                format!(" {}", field.value)
            };

            let input_frame_width = 31;
            let truncated_val: String = val_to_show.chars().take(input_frame_width - 2).collect();

            if is_focused {
                out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                print!("[");
                out.queue(SetForegroundColor(Color::White)).unwrap();
                print!("{:<width$}", truncated_val, width = input_frame_width - 2);
                out.queue(SetForegroundColor(Color::Cyan)).unwrap();
                print!("]");
            } else {
                out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                print!(" ");
                print!("{:<width$}", truncated_val, width = input_frame_width - 2);
                print!(" ");
            }
            out.queue(ResetColor).unwrap();
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

        let btn_y = start_y + 14;
        
        out.queue(crossterm::cursor::MoveTo(start_x + 12, btn_y)).unwrap();
        if active_idx == 6 {
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
        if active_idx == 7 {
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
                match validate_and_build(&fields) {
                    Ok(conn) => break Ok(Some(conn)),
                    Err(e) => {
                        error_msg = Some(e);
                        continue;
                    }
                }
            }

            match key.code {
                KeyCode::Up => {
                    active_idx = if active_idx == 0 { 7 } else { active_idx - 1 };
                    error_msg = None;
                }
                KeyCode::Down | KeyCode::Tab => {
                    active_idx = if active_idx == 7 { 0 } else { active_idx + 1 };
                    error_msg = None;
                }
                KeyCode::BackTab => {
                    active_idx = if active_idx == 0 { 7 } else { active_idx - 1 };
                    error_msg = None;
                }
                KeyCode::Left => {
                    if active_idx == 7 {
                        active_idx = 6;
                    } else if active_idx == 6 {
                        active_idx = 7;
                    }
                }
                KeyCode::Right => {
                    if active_idx == 6 {
                        active_idx = 7;
                    } else if active_idx == 7 {
                        active_idx = 6;
                    }
                }
                KeyCode::Enter => {
                    if active_idx < 6 {
                        active_idx += 1;
                    } else if active_idx == 6 {
                        match validate_and_build(&fields) {
                            Ok(conn) => break Ok(Some(conn)),
                            Err(e) => {
                                error_msg = Some(e);
                            }
                        }
                    } else if active_idx == 7 {
                        break Ok(None);
                    }
                }
                KeyCode::Backspace => {
                    if active_idx < 6 {
                        fields[active_idx].value.pop();
                        error_msg = None;
                    }
                }
                KeyCode::Char(c) => {
                    if active_idx < 6 {
                        fields[active_idx].value.push(c);
                        error_msg = None;
                    }
                }
                _ => {}
            }
        }
    };

    result
}

fn validate_and_build(fields: &[FormField]) -> Result<ServerConnection, String> {
    let nickname = fields[0].value.trim().to_string();
    let host = fields[1].value.trim().to_string();
    let port_str = fields[2].value.trim();
    let username = fields[3].value.trim().to_string();
    let password = fields[4].value.trim().to_string();
    let ssh_key = fields[5].value.trim().to_string();

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

    Ok(ServerConnection {
        nickname,
        host,
        port,
        username,
        password: opt_password,
        ssh_key: opt_ssh_key,
    })
}
