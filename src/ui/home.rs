use crate::app::{App, HOME_ITEMS, Screen};
use crate::ui::components::{chrome, list_block, render_ascii_logo};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn render(frame: &mut Frame, app: &App) {
    let subtitle = app
        .vault
        .as_ref()
        .map(|v| format!("{} wallets", v.wallets().len()))
        .unwrap_or_default();
    let body = chrome(
        frame,
        app,
        "Home",
        &subtitle,
        "`↑/↓` navigate  ·  `Enter` select  ·  `q` quit",
    );
    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Min(8),
        ])
        .split(body);
    render_ascii_logo(frame, body_chunks[1]);

    let menu_area = centered_menu_rect(body_chunks[3]);
    let items: Vec<String> = HOME_ITEMS.iter().map(|s| s.to_string()).collect();
    list_block(frame, menu_area, "Menu", &items, app.home.selected, true);
}

fn centered_menu_rect(area: Rect) -> Rect {
    let desired_height = (HOME_ITEMS.len() as u16) + 7;
    let height = desired_height.min(area.height);
    Rect {
        x: area.x + area.width.saturating_sub(area.width * 40 / 100) / 2,
        y: area.y,
        width: area.width * 40 / 100,
        height,
    }
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.quit = true,
        KeyCode::Up if app.home.selected > 0 => {
            app.home.selected -= 1;
        }
        KeyCode::Down if app.home.selected + 1 < HOME_ITEMS.len() => {
            app.home.selected += 1;
        }
        KeyCode::Enter => match app.home.selected {
            0 => app.screen = Screen::Wallets,
            1 => app.screen = Screen::SingleSwap,
            2 => app.screen = Screen::MultiSwap,
            3 => {
                if app.status.order_id.is_some() {
                    app.screen = Screen::OrderStatus;
                } else {
                    app.toast_info("no order yet — submit a swap first");
                }
            }
            4 => app.open_account_screen(),
            5 => app.quit = true,
            _ => {}
        },
        _ => {}
    }
}
