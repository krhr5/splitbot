use crate::app::{App, Toast, ToastLevel};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

const SPLITBOT_ASCII: &[&str] = &[
    "  ___      _ _ _   ___      _   ",
    " / __|_ __| (_) |_| _ ) ___| |_ ",
    " \\__ \\ '_ \\ | |  _| _ \\/ _ \\  _|",
    " |___/ .__/_|_|\\__|___/\\___/\\__|",
    "     |_|                        ",
];

pub fn accent_color() -> Color {
    Color::Indexed(80)
}

pub fn accent_fill_color() -> Color {
    Color::Indexed(116)
}

pub fn logo_color() -> Color {
    Color::Indexed(116)
}

pub fn accent() -> Style {
    Style::default()
        .fg(accent_color())
        .add_modifier(Modifier::BOLD)
}

pub fn muted() -> Style {
    Style::default().fg(Color::DarkGray)
}

pub fn focus_border(focused: bool) -> Style {
    if focused {
        Style::default().fg(accent_color())
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

pub fn field_label_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(accent_color())
    } else {
        Style::default().fg(Color::Yellow)
    }
}

pub fn draw_header(frame: &mut Frame, area: Rect, title: &str, subtitle: &str) {
    let title_line = Line::from(vec![
        Span::styled(
            "  SplitBOT  ",
            Style::default()
                .bg(accent_fill_color())
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(title, accent()),
        Span::raw("  "),
        Span::styled(subtitle, muted()),
    ]);
    let p = Paragraph::new(title_line).alignment(Alignment::Left);
    frame.render_widget(p, area);
}

pub fn draw_status(frame: &mut Frame, area: Rect, toast: Option<&Toast>) {
    if let Some(t) = toast {
        let style = match t.level {
            ToastLevel::Info => Style::default().fg(Color::White),
            ToastLevel::Success => Style::default().fg(Color::Green),
            ToastLevel::Error => Style::default().fg(Color::Red),
        };
        let p = Paragraph::new(Line::from(Span::styled(t.message.clone(), style)))
            .alignment(Alignment::Left);
        frame.render_widget(p, area);
    }
}

pub fn draw_hints(frame: &mut Frame, area: Rect, hints: &str) {
    let p = Paragraph::new(parse_hint_line(hints))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    frame.render_widget(p, area);
}

fn hint_line_count(width: u16, hints: &str) -> u16 {
    let available = width.max(1) as usize;
    let visible_width = hints.chars().filter(|&ch| ch != '`').count().max(1);
    visible_width.div_ceil(available) as u16
}

fn parse_hint_line(hints: &str) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut buf = String::new();
    let mut highlighted = false;

    for ch in hints.chars() {
        if ch == '`' {
            if !buf.is_empty() {
                let text = std::mem::take(&mut buf);
                let style = if highlighted {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    muted()
                };
                spans.push(Span::styled(text, style));
            }
            highlighted = !highlighted;
        } else {
            buf.push(ch);
        }
    }

    if !buf.is_empty() {
        let style = if highlighted {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            muted()
        };
        spans.push(Span::styled(buf, style));
    }

    Line::from(spans)
}

pub fn chrome(frame: &mut Frame, app: &App, title: &str, subtitle: &str, hints: &str) -> Rect {
    let hint_height = hint_line_count(frame.area().width, hints);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
            Constraint::Length(hint_height),
        ])
        .split(frame.area());
    draw_header(frame, chunks[0], title, subtitle);
    draw_status(frame, chunks[2], app.toast.as_ref());
    draw_hints(frame, chunks[3], hints);
    chunks[1]
}

#[cfg(test)]
mod tests {
    use super::hint_line_count;

    #[test]
    fn hint_line_count_collapses_when_line_fits() {
        assert_eq!(hint_line_count(80, "`q` quit"), 1);
    }

    #[test]
    fn hint_line_count_expands_when_line_wraps() {
        assert!(hint_line_count(10, "`Tab`: next field  ·  `Esc`: back") > 1);
    }
}

pub fn render_ascii_logo(frame: &mut Frame, area: Rect) {
    if area.width < 36 || area.height < SPLITBOT_ASCII.len() as u16 {
        return;
    }

    let lines: Vec<Line> = SPLITBOT_ASCII
        .iter()
        .map(|line| {
            Line::from(Span::styled(
                (*line).to_string(),
                Style::default()
                    .fg(logo_color())
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

pub fn input_block(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    focused: bool,
    mask: bool,
) {
    let display = if mask {
        "*".repeat(value.chars().count())
    } else {
        value.to_string()
    };
    let cursor = if focused { "▏" } else { " " };
    let line = Line::from(vec![
        Span::raw(display),
        Span::styled(cursor, Style::default().fg(accent_color())),
    ]);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(focus_border(focused))
        .title(Line::from(Span::styled(
            format!(" {label} "),
            field_label_style(focused),
        )));
    let p = Paragraph::new(line).block(block);
    frame.render_widget(p, area);
}

pub fn list_block(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    items: &[String],
    selected: usize,
    focused: bool,
) {
    let lines: Vec<Line> = items
        .iter()
        .enumerate()
        .map(|(i, s)| {
            if i == selected {
                Line::from(Span::styled(
                    format!("› {s}"),
                    Style::default()
                        .fg(Color::Black)
                        .bg(accent_fill_color())
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(Span::raw(format!("  {s}")))
            }
        })
        .collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(focus_border(focused))
        .title(Line::from(Span::styled(
            format!(" {label} "),
            field_label_style(focused),
        )));
    let p = Paragraph::new(lines).block(block);
    frame.render_widget(p, area);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
