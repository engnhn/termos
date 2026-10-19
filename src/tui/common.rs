use crossterm::{
    QueueableCommand,
    cursor::{self, Hide, Show},
    execute,
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::io::{Write, stdout};

pub struct TerminalGuard;

pub fn safe_display(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for ch in input.chars() {
        let code = ch as u32;
        let bidi_override =
            matches!(code, 0x061C | 0x200E | 0x200F | 0x202A..=0x202E | 0x2066..=0x2069);
        if ch.is_control() || bidi_override {
            output.extend(ch.escape_unicode());
        } else {
            output.push(ch);
        }
    }
    output
}

impl TerminalGuard {
    pub fn create() -> Result<Self, String> {
        enable_raw_mode().map_err(|e| format!("Could not enable raw mode: {}", e))?;
        execute!(stdout(), EnterAlternateScreen, Hide)
            .map_err(|e| format!("Could not start TUI: {}", e))?;
        Ok(TerminalGuard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(stdout(), LeaveAlternateScreen, Show);
        let _ = disable_raw_mode();
    }
}

pub fn check_size_or_draw_error(
    out: &mut std::io::Stdout,
    box_width: u16,
    box_height: u16,
) -> Result<Option<(u16, u16)>, String> {
    let (cols, rows) =
        crossterm::terminal::size().map_err(|e| format!("Could not get terminal size: {}", e))?;
    if cols < box_width + 2 || rows < box_height + 2 {
        let msg = "Please enlarge your terminal screen!";
        let x = cols.saturating_sub(msg.chars().count() as u16) / 2;
        let y = rows / 2;
        out.queue(cursor::MoveTo(x, y)).unwrap();
        out.queue(SetForegroundColor(Color::Red)).unwrap();
        out.queue(SetAttribute(Attribute::Bold)).unwrap();
        print!("{}", msg);
        out.queue(ResetColor).unwrap();
        out.queue(SetAttribute(Attribute::Reset)).unwrap();
        let _ = out.flush();
        Ok(None)
    } else {
        Ok(Some((cols, rows)))
    }
}

pub fn draw_box(
    out: &mut std::io::Stdout,
    start_x: u16,
    start_y: u16,
    width: u16,
    height: u16,
    title: &str,
) {
    out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();

    out.queue(cursor::MoveTo(start_x, start_y)).unwrap();
    let border_chars = width - 2;
    let title = safe_display(title);
    let title: String = title
        .chars()
        .take(border_chars.saturating_sub(2) as usize)
        .collect();
    let header_len = title.chars().count() as u16;
    let left_border_len = (border_chars - header_len) / 2;
    let right_border_len = border_chars - header_len - left_border_len;

    print!("┌");
    print!("{}", "─".repeat(left_border_len as usize));
    out.queue(SetForegroundColor(Color::Cyan)).unwrap();
    out.queue(SetAttribute(Attribute::Bold)).unwrap();
    print!("{}", title);
    out.queue(ResetColor).unwrap();
    out.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
    print!("{}", "─".repeat(right_border_len as usize));
    print!("┐");

    for y in (start_y + 1)..(start_y + height - 5) {
        out.queue(cursor::MoveTo(start_x, y)).unwrap();
        print!("│");
        out.queue(cursor::MoveTo(start_x + width - 1, y)).unwrap();
        print!("│");
    }

    let div_y = start_y + height - 5;
    out.queue(cursor::MoveTo(start_x, div_y)).unwrap();
    print!("├");
    print!("{}", "─".repeat((width - 2) as usize));
    print!("┤");

    for y in (div_y + 1)..(start_y + height - 1) {
        out.queue(cursor::MoveTo(start_x, y)).unwrap();
        print!("│");
        out.queue(cursor::MoveTo(start_x + width - 1, y)).unwrap();
        print!("│");
    }

    out.queue(cursor::MoveTo(start_x, start_y + height - 1))
        .unwrap();
    print!("└");
    print!("{}", "─".repeat((width - 2) as usize));
    print!("┘");

    out.queue(ResetColor).unwrap();
    out.queue(SetAttribute(Attribute::Reset)).unwrap();
}

#[cfg(test)]
mod tests {
    use super::safe_display;

    #[test]
    fn terminal_control_and_bidi_characters_are_escaped() {
        assert_eq!(safe_display("ok\x1b[2J"), "ok\\u{1b}[2J");
        assert_eq!(safe_display("a\u{202e}b"), "a\\u{202e}b");
    }
}
