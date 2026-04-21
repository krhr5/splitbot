pub mod account;
pub mod components;
pub mod home;
pub mod multiswap;
pub mod status;
pub mod swap;
pub mod unlock;
pub mod wallets;

use crate::app::{App, Screen};
use crossterm::event::KeyEvent;
use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::Unlock => unlock::render(frame, app),
        Screen::Home => home::render(frame, app),
        Screen::Account => account::render(frame, app),
        Screen::Wallets => wallets::render(frame, app),
        Screen::SingleSwap => swap::render(frame, app),
        Screen::MultiSwap => multiswap::render(frame, app),
        Screen::OrderStatus => status::render(frame, app),
    }
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match app.screen {
        Screen::Unlock => unlock::handle_key(app, key),
        Screen::Home => home::handle_key(app, key),
        Screen::Account => account::handle_key(app, key),
        Screen::Wallets => wallets::handle_key(app, key),
        Screen::SingleSwap => swap::handle_key(app, key),
        Screen::MultiSwap => multiswap::handle_key(app, key),
        Screen::OrderStatus => status::handle_key(app, key),
    }
}
