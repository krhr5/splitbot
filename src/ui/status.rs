use crate::app::{App, Screen};
use crate::events::AppEvent;
use crate::ui::components::{accent_color, accent_fill_color, chrome, focus_border};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use splitnow::{Order, OrderLegStatus, OrderStatus};
use std::collections::HashSet;
use std::time::{Duration, Instant};

pub fn render(frame: &mut Frame, app: &App) {
    let hints = if app.status.show_raw {
        "`r` refresh  ·  `d` hide raw  ·  `Esc` back"
    } else {
        "`r` refresh  ·  `d` show raw  ·  `Esc` back"
    };
    let subtitle = app
        .status
        .order_id
        .clone()
        .unwrap_or_else(|| "no order".into());
    let body = chrome(frame, app, "Order Status", &subtitle, hints);

    if app.status.show_raw {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(10), Constraint::Min(3)])
            .split(body);
        render_summary(frame, chunks[0], app);
        render_details(frame, chunks[1], app);
    } else {
        render_summary(frame, body, app);
    }
}

fn render_summary(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(focus_border(true))
        .title(" summary ");
    let mut lines: Vec<Line> = Vec::new();
    match (&app.status.order_id, &app.status.latest) {
        (Some(id), Some(order)) => {
            lines.push(Line::from(vec![
                Span::styled("order id  ", Style::default().fg(Color::DarkGray)),
                Span::styled(id.clone(), Style::default().add_modifier(Modifier::BOLD)),
            ]));
            if let (Some(amount), Some(address)) = (
                app.status.deposit_amount,
                app.status.deposit_address.as_ref(),
            ) {
                lines.push(Line::from(vec![
                    Span::styled("deposit   ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format!("{amount} to {address}")),
                ]));
            }
            lines.push(Line::from(vec![
                Span::styled("status    ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:?}", order.status_short),
                    Style::default().fg(accent_color()),
                ),
                Span::raw(format!("  ({})", order.status_text)),
            ]));
            lines.push(progress_line(order, area.width.saturating_sub(2)));
            lines.push(wallets_line(order));
            if let Some(t) = app.status.last_poll {
                let elapsed = Instant::now().saturating_duration_since(t);
                lines.push(Line::from(Span::styled(
                    format!("last refresh: {:.1}s ago", elapsed.as_secs_f32()),
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }
        (Some(id), None) => {
            lines.push(Line::from(Span::styled(
                format!("polling {id}…"),
                Style::default().fg(Color::Yellow),
            )));
        }
        _ => {
            lines.push(Line::from(Span::styled(
                "no order — submit a swap first",
                Style::default().fg(Color::DarkGray),
            )));
        }
    }
    if let Some(err) = &app.status.error {
        lines.push(Line::from(Span::styled(
            if app.status.latest.is_some() {
                format!("refresh error: {err}")
            } else {
                err.clone()
            },
            if app.status.latest.is_some() {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Red)
            },
        )));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(block),
        area,
    );
}

fn render_details(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(focus_border(false))
        .title(" raw status ");
    let content: String = app
        .status
        .latest
        .as_ref()
        .map(|order| format!("{order:#?}"))
        .unwrap_or_else(|| "waiting for first response…".into());
    frame.render_widget(Paragraph::new(content).block(block), area);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.screen = Screen::Home,
        KeyCode::Char('r') => poll_now(app),
        KeyCode::Char('d') => app.status.show_raw = !app.status.show_raw,
        _ => {}
    }
}

pub fn maybe_poll(app: &mut App) {
    if let Some(order) = app.status.latest.as_ref()
        && is_terminal(order)
    {
        return;
    }
    let due = match app.status.last_poll {
        Some(t) => Instant::now().saturating_duration_since(t) >= Duration::from_secs(5),
        None => true,
    };
    if due && app.status.order_id.is_some() {
        poll_now(app);
    }
}

fn poll_now(app: &mut App) {
    let Some(client) = app.client.clone() else {
        return;
    };
    let Some(id) = app.status.order_id.clone() else {
        return;
    };
    app.status.last_poll = Some(Instant::now());
    let tx = app.tx.clone();
    tokio::spawn(async move {
        let r = client.order_details(id).await.map_err(|e| e.to_string());
        let _ = tx.send(AppEvent::StatusTick(Box::new(r)));
    });
}

pub fn is_terminal(order: &Order) -> bool {
    matches!(
        order.status_short,
        OrderStatus::Completed
            | OrderStatus::Expired
            | OrderStatus::Halted
            | OrderStatus::Failed
            | OrderStatus::Refunded
    )
}

pub fn progress_line(order: &Order, available_width: u16) -> Line<'static> {
    let color = match order.status_short {
        OrderStatus::Completed => Color::Green,
        OrderStatus::Expired | OrderStatus::Refunded => Color::Yellow,
        OrderStatus::Halted | OrderStatus::Failed => Color::Red,
        _ => accent_fill_color(),
    };
    let suffix = format!("  {}", order.status_text);
    let label = "progress  ";
    let min_bar_width = 10usize;
    let bar_width = if available_width as usize > label.len() + suffix.len() + min_bar_width {
        available_width as usize - label.len() - suffix.len()
    } else {
        min_bar_width
    };

    let mut spans = vec![Span::styled(
        "progress  ",
        Style::default().fg(Color::DarkGray),
    )];
    spans.extend(braille_bar_spans(
        progress_ratio(order),
        bar_width,
        color,
        Color::DarkGray,
    ));
    spans.push(Span::raw(suffix));
    Line::from(spans)
}

fn progress_ratio(order: &Order) -> f64 {
    match order.status_short {
        OrderStatus::Pending => 0.25,
        OrderStatus::Sending => 0.50,
        OrderStatus::Monitoring => 0.75,
        OrderStatus::Completed
        | OrderStatus::Expired
        | OrderStatus::Halted
        | OrderStatus::Failed
        | OrderStatus::Refunded => 1.0,
    }
}

fn braille_bar_spans(
    progress: f64,
    width: usize,
    fill_color: Color,
    empty_color: Color,
) -> Vec<Span<'static>> {
    let width = width.max(4);
    let body_width = width.saturating_sub(2).max(1);
    let progress = progress.clamp(0.0, 1.0);
    let filled = if progress <= 0.0 {
        0
    } else {
        ((progress * body_width as f64).round() as usize).clamp(1, body_width)
    };
    let empty = body_width.saturating_sub(filled);
    let left_style = if filled > 0 {
        Style::default().fg(fill_color)
    } else {
        Style::default().fg(empty_color)
    };
    let right_style = if filled == body_width {
        Style::default().fg(fill_color)
    } else {
        Style::default().fg(empty_color)
    };

    let mut spans = Vec::with_capacity(5);
    spans.push(Span::styled("⢾", left_style));
    if filled > 0 {
        spans.push(Span::styled(
            "⣿".repeat(filled),
            Style::default().fg(fill_color),
        ));
    }
    if empty > 0 {
        spans.push(Span::styled(
            "⣀".repeat(empty),
            Style::default().fg(empty_color),
        ));
    }
    spans.push(Span::styled("⡷", right_style));
    spans
}

pub fn wallets_line(order: &Order) -> Line<'static> {
    let total = order.order_outputs.len();
    let completed = order
        .order_legs
        .iter()
        .filter(|leg| leg.status_short == OrderLegStatus::Completed)
        .map(|leg| leg.order_leg_output.to_distribution_id)
        .collect::<HashSet<_>>()
        .len();
    let style = if total > 0 && completed == total {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    Line::from(vec![
        Span::styled("wallets   ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{completed}/{total} completed"), style),
    ])
}

#[cfg(test)]
mod tests {
    use super::braille_bar_spans;
    use ratatui::style::Color;
    use ratatui::text::Span;

    fn flatten(spans: Vec<Span<'static>>) -> String {
        spans
            .into_iter()
            .map(|span| span.content.into_owned())
            .collect::<Vec<_>>()
            .join("")
    }

    #[test]
    fn braille_bar_renders_empty_meter() {
        let bar = flatten(braille_bar_spans(0.0, 8, Color::Blue, Color::DarkGray));
        assert_eq!(bar, "⢾⣀⣀⣀⣀⣀⣀⡷");
    }

    #[test]
    fn braille_bar_renders_partial_meter() {
        let bar = flatten(braille_bar_spans(0.5, 8, Color::Blue, Color::DarkGray));
        assert_eq!(bar, "⢾⣿⣿⣿⣀⣀⣀⡷");
    }

    #[test]
    fn braille_bar_renders_full_meter() {
        let bar = flatten(braille_bar_spans(1.0, 8, Color::Blue, Color::DarkGray));
        assert_eq!(bar, "⢾⣿⣿⣿⣿⣿⣿⡷");
    }
}
