use crate::account;
use crate::app::{App, Screen, spawn_splitnow_bootstrap};
use crate::config;
use crate::splitnow::SplitnowClient;
use crate::ui::components::{chrome, input_block, muted};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use tui_input::backend::crossterm::EventHandler;

pub fn render(frame: &mut Frame, app: &App) {
    let subtitle = app
        .vault
        .as_ref()
        .map(|vault| format!("{} wallets", vault.wallets().len()))
        .unwrap_or_default();
    let hints = if app.account.editing_api_key {
        "`Enter`: save API key  ·  typing: edit  ·  `Esc`: cancel"
    } else {
        "`r`: refresh balances  ·  `e`: edit API key  ·  `v`: reveal/hide key  ·  `Esc`: back"
    };
    let body = chrome(frame, app, "Account", &subtitle, hints);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),
            Constraint::Length(5),
            Constraint::Min(5),
        ])
        .split(body);

    frame.render_widget(account_summary(app), chunks[0]);
    render_api_key(frame, app, chunks[1]);
    frame.render_widget(account_notes(), chunks[2]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    if app.account.editing_api_key {
        handle_api_key_edit(app, key);
        return;
    }

    match key.code {
        KeyCode::Esc => app.screen = Screen::Home,
        KeyCode::Char('r') | KeyCode::Char('R') => account::refresh_balances(app),
        KeyCode::Char('e') | KeyCode::Char('E') => {
            app.sync_account_api_key_input();
            app.account.editing_api_key = true;
        }
        KeyCode::Char('v') | KeyCode::Char('V') => {
            app.account.reveal_api_key = !app.account.reveal_api_key;
        }
        _ => {}
    }
}

fn handle_api_key_edit(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.account.editing_api_key = false;
            app.sync_account_api_key_input();
        }
        KeyCode::Enter => save_api_key(app),
        _ => {
            let evt = crossterm::event::Event::Key(key);
            app.account.api_key_input.handle_event(&evt);
        }
    }
}

fn save_api_key(app: &mut App) {
    let api_key = app.account.api_key_input.value().trim().to_string();
    if api_key.is_empty() {
        app.account.api_key_error = Some("API key cannot be empty".into());
        return;
    }

    let client = match SplitnowClient::new(api_key.clone()) {
        Ok(client) => client,
        Err(e) => {
            app.account.api_key_error = Some(format!("client init: {e}"));
            return;
        }
    };

    let Some(vault) = app.vault.as_mut() else {
        app.account.api_key_error = Some("vault not loaded".into());
        return;
    };

    if let Err(e) = vault.set_api_key(api_key) {
        app.account.api_key_error = Some(format!("save API key: {e}"));
        return;
    }

    app.client = Some(client.clone());
    app.account.editing_api_key = false;
    app.account.reveal_api_key = false;
    app.account.api_key_error = None;
    spawn_splitnow_bootstrap(app, client);
    app.toast_ok("SplitNOW API key updated");
}

fn account_summary(app: &App) -> Paragraph<'static> {
    let wallet_count = app
        .vault
        .as_ref()
        .map(|vault| vault.wallets().len())
        .unwrap_or(0);
    let data_location = config::app_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "unavailable".into());
    let total_sol = app
        .account
        .total_sol
        .map(|value| format!("{value:.6} SOL"))
        .unwrap_or_else(|| "unavailable".into());
    let total_usd = app
        .account
        .total_usd
        .map(|value| format!("${value:.2} USD"))
        .unwrap_or_else(|| "USD unavailable".into());

    let mut lines = vec![
        Line::from(vec![
            Span::styled("wallets: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(wallet_count.to_string()),
        ]),
        Line::from(vec![
            Span::styled(
                "native SOL total: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(total_sol),
        ]),
        Line::from(vec![
            Span::styled(
                "USD estimate: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(total_usd),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "data location:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(data_location, muted())),
    ];

    if app.account.balance_loading {
        lines.push(Line::from(Span::styled(
            "refreshing balances…",
            Style::default().fg(Color::Yellow),
        )));
    } else if let Some(err) = &app.account.balance_error {
        lines.push(Line::from(Span::styled(
            format!("balance error: {err}"),
            Style::default().fg(Color::Red),
        )));
    }

    Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title(" overview "))
}

fn render_api_key(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    if app.account.editing_api_key {
        input_block(
            frame,
            area,
            "SplitNOW API key",
            app.account.api_key_input.value(),
            true,
            false,
        );
        if let Some(err) = &app.account.api_key_error {
            let overlay = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(area);
            let err_line = Paragraph::new(Line::from(Span::styled(
                err.clone(),
                Style::default().fg(Color::Red),
            )));
            frame.render_widget(err_line, overlay[1]);
        }
        return;
    }

    let stored_key = app
        .vault
        .as_ref()
        .and_then(|vault| vault.api_key())
        .unwrap_or_default();
    let display_key = if stored_key.is_empty() {
        "not set".to_string()
    } else if app.account.reveal_api_key {
        stored_key.to_string()
    } else {
        mask_api_key(stored_key)
    };
    let lines = vec![
        Line::from(vec![
            Span::styled("status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(if stored_key.is_empty() {
                "missing"
            } else {
                "configured"
            }),
        ]),
        Line::from(vec![
            Span::styled("key: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(display_key, muted()),
        ]),
    ];
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" SplitNOW API ");
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }).block(block),
        area,
    );
}

fn account_notes() -> Paragraph<'static> {
    let lines = vec![
        Line::from(Span::styled(
            "Balances are summed from native SOL across the wallets stored in this vault.",
            muted(),
        )),
        Line::from(Span::styled(
            "USD is a live estimate using the current SOL market price.",
            muted(),
        )),
    ];
    Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title(" notes "))
}

fn mask_api_key(value: &str) -> String {
    if value.chars().count() <= 10 {
        return "********".into();
    }

    let start: String = value.chars().take(6).collect();
    let end: String = value
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{start}…{end}")
}
