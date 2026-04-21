use crate::app::{App, Screen, UnlockStage};
use crate::config;
use crate::splitnow::SplitnowClient;
use crate::ui::components::{centered_rect, chrome, input_block, muted, render_ascii_logo};
use crate::vault::Vault;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use tui_input::backend::crossterm::EventHandler;

fn chunk_text(value: &str, width: u16) -> Vec<String> {
    let width = usize::from(width.max(1));
    let chars: Vec<char> = value.chars().collect();
    chars
        .chunks(width)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect()
}

pub fn render(frame: &mut Frame, app: &App) {
    let (title, hints) = if app.unlock.first_run {
        ("Create vault", "`Enter`: next  ·  `Esc`: quit")
    } else {
        ("Unlock vault", "`Enter`: unlock  ·  `Esc`: quit")
    };
    let body = chrome(frame, app, title, "", hints);
    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Length(2),
            Constraint::Min(12),
        ])
        .split(body);
    render_ascii_logo(frame, body_chunks[1]);

    let area = centered_rect(60, 100, body_chunks[3]);

    let chunks =
        if app.unlock.first_run && matches!(app.unlock.stage, UnlockStage::ConfirmPassphrase) {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(2),
                    Constraint::Length(1),
                ])
                .split(area)
        } else if app.unlock.first_run {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(2),
                    Constraint::Length(1),
                ])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(area)
        };

    let banner_text = match app.unlock.stage {
        UnlockStage::EnterPassphrase => "vault found — enter passphrase",
        UnlockStage::NewPassphrase => "set a master passphrase",
        UnlockStage::ConfirmPassphrase => "confirm passphrase",
        UnlockStage::NewApiKey => "paste your SplitNOW API key",
    };
    let banner = Paragraph::new(Line::from(Span::styled(banner_text, muted())))
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(banner, chunks[0]);

    match app.unlock.stage {
        UnlockStage::EnterPassphrase | UnlockStage::NewPassphrase => {
            input_block(
                frame,
                chunks[1],
                "passphrase",
                app.unlock.passphrase.value(),
                true,
                true,
            );
        }
        UnlockStage::ConfirmPassphrase => {
            input_block(
                frame,
                chunks[1],
                "passphrase",
                app.unlock.passphrase.value(),
                false,
                true,
            );
            input_block(
                frame,
                chunks[2],
                "confirm",
                app.unlock.confirm.value(),
                true,
                true,
            );
            let warning = Paragraph::new(Line::from(Span::styled(
                "Store this passphrase securely. You will not be able to view or edit it later.",
                Style::default().fg(ratatui::style::Color::Red),
            )))
            .wrap(Wrap { trim: true });
            frame.render_widget(warning, chunks[3]);
        }
        UnlockStage::NewApiKey => {
            input_block(
                frame,
                chunks[1],
                "SplitNOW API key",
                app.unlock.api_key.value(),
                true,
                false,
            );
        }
    }

    if app.unlock.first_run {
        let data_lines = match config::app_dir() {
            Ok(path) => {
                let mut lines = vec![Line::from(Span::styled("data location:", muted()))];
                for chunk in chunk_text(
                    &path.display().to_string(),
                    chunks[data_chunk_index(app)].width,
                ) {
                    lines.push(Line::from(Span::styled(chunk, muted())));
                }
                lines
            }
            Err(_) => vec![Line::from(Span::styled(
                "data location: unavailable",
                muted(),
            ))],
        };
        let data_chunk = chunks[data_chunk_index(app)];
        let data_line = Paragraph::new(data_lines).wrap(Wrap { trim: true });
        frame.render_widget(data_line, data_chunk);
    }

    if let Some(err) = &app.unlock.error {
        let p = Paragraph::new(Line::from(Span::styled(
            err.clone(),
            Style::default().fg(ratatui::style::Color::Red),
        )));
        let error_chunk = if matches!(app.unlock.stage, UnlockStage::ConfirmPassphrase) {
            chunks[5]
        } else {
            chunks[3]
        };
        frame.render_widget(p, error_chunk);
    }
}

fn data_chunk_index(app: &App) -> usize {
    if matches!(app.unlock.stage, UnlockStage::ConfirmPassphrase) {
        4
    } else {
        2
    }
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Esc
        || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c'))
    {
        app.quit = true;
        return;
    }

    match app.unlock.stage {
        UnlockStage::EnterPassphrase => handle_enter_passphrase(app, key),
        UnlockStage::NewPassphrase => handle_new_passphrase(app, key),
        UnlockStage::ConfirmPassphrase => handle_confirm_passphrase(app, key),
        UnlockStage::NewApiKey => handle_new_api_key(app, key),
    }
}

fn forward_input(input: &mut tui_input::Input, key: KeyEvent) {
    let evt = crossterm::event::Event::Key(key);
    input.handle_event(&evt);
}

fn handle_enter_passphrase(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Enter {
        let path = match config::vault_path() {
            Ok(p) => p,
            Err(e) => {
                app.unlock.error = Some(e.to_string());
                return;
            }
        };
        let pass = app.unlock.passphrase.value().to_string();
        match Vault::unlock(path, pass) {
            Ok(v) => {
                let api_key = v.api_key().map(|s| s.to_string());
                app.vault = Some(v);
                if let Some(k) = api_key {
                    match SplitnowClient::new(k) {
                        Ok(c) => {
                            crate::app::spawn_splitnow_bootstrap(app, c.clone());
                            app.client = Some(c);
                        }
                        Err(e) => app.toast_err(format!("client init: {e}")),
                    }
                }
                app.restore_last_order();
                app.screen = Screen::Home;
            }
            Err(e) => {
                let full = format!("{e:#}");
                if full.contains("wrong passphrase") || full.contains("decrypt failed") {
                    app.unlock.error = Some("wrong passphrase".into());
                } else {
                    app.unlock.error = Some(e.to_string());
                }
            }
        }
        return;
    }
    forward_input(&mut app.unlock.passphrase, key);
}

fn handle_new_passphrase(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Enter {
        if app.unlock.passphrase.value().is_empty() {
            app.unlock.error = Some("passphrase cannot be empty".into());
            return;
        }
        app.unlock.stage = UnlockStage::ConfirmPassphrase;
        app.unlock.error = None;
        return;
    }
    forward_input(&mut app.unlock.passphrase, key);
}

fn handle_confirm_passphrase(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Enter {
        if app.unlock.passphrase.value() != app.unlock.confirm.value() {
            app.unlock.error = Some("passphrases do not match".into());
            app.unlock.confirm.reset();
            return;
        }
        app.unlock.stage = UnlockStage::NewApiKey;
        app.unlock.error = None;
        return;
    }
    forward_input(&mut app.unlock.confirm, key);
}

fn handle_new_api_key(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Enter {
        let api_key = app.unlock.api_key.value().trim().to_string();
        if api_key.is_empty() {
            app.unlock.error = Some("API key cannot be empty".into());
            return;
        }
        let path = match config::vault_path() {
            Ok(p) => p,
            Err(e) => {
                app.unlock.error = Some(e.to_string());
                return;
            }
        };
        let pass = app.unlock.passphrase.value().to_string();
        match Vault::create(path, pass, api_key.clone()) {
            Ok(v) => {
                app.vault = Some(v);
                match SplitnowClient::new(api_key) {
                    Ok(c) => {
                        crate::app::spawn_splitnow_bootstrap(app, c.clone());
                        app.client = Some(c);
                    }
                    Err(e) => app.toast_err(format!("client init: {e}")),
                }
                app.screen = Screen::Home;
            }
            Err(e) => app.unlock.error = Some(e.to_string()),
        }
        return;
    }
    forward_input(&mut app.unlock.api_key, key);
}
