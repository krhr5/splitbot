mod account;
mod app;
mod config;
mod events;
mod persist;
mod splitnow;
mod ui;
mod vault;
mod wallet;

use crate::app::App;
use crate::events::{AppEvent, channel};
use anyhow::Result;
use crossterm::event::{Event as CtEvent, EventStream};
use crossterm::terminal::SetSize;
use futures::StreamExt;
use std::io::stdout;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install().ok();
    init_logging()?;
    install_panic_hook();

    let (tx, mut rx) = channel();
    let vault_path = config::vault_path()?;
    let first_run = !vault::Vault::exists(&vault_path);
    let mut app = App::new(first_run, tx.clone());

    // periodic tick for toast GC + status polling
    let tick_tx = tx.clone();
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_millis(500));
        loop {
            ticker.tick().await;
            if tick_tx.send(AppEvent::Tick).is_err() {
                break;
            }
        }
    });

    let mut terminal = ratatui::init();
    let _ = crossterm::execute!(stdout(), SetSize(105, 30));
    let mut term_stream = EventStream::new();

    let run_result: Result<()> = async {
        loop {
            terminal.draw(|frame| ui::render(frame, &app))?;
            tokio::select! {
                maybe_term = term_stream.next() => {
                    if let Some(Ok(evt)) = maybe_term {
                        handle_terminal_event(&mut app, evt);
                    }
                }
                maybe_evt = rx.recv() => {
                    if let Some(e) = maybe_evt {
                        on_event(&mut app, e);
                    }
                }
            }
            if app.quit {
                break;
            }
        }
        Ok(())
    }
    .await;

    ratatui::restore();
    run_result
}

fn handle_terminal_event(app: &mut App, evt: CtEvent) {
    match evt {
        CtEvent::Key(k) => ui::handle_key(app, k),
        CtEvent::Resize(_, _) => {}
        _ => {}
    }
}

fn on_event(app: &mut App, evt: AppEvent) {
    match evt {
        AppEvent::Tick => {
            app.gc_toast();
            ui::status::maybe_poll(app);
        }
        other => app::dispatch_app_event(app, other),
    }
}

fn init_logging() -> Result<()> {
    let path = config::log_path()?;
    let dir = path.parent().unwrap().to_path_buf();
    let file_appender = tracing_appender::rolling::never(dir, config::LOG_FILE);
    let (nb, guard) = tracing_appender::non_blocking(file_appender);
    std::mem::forget(guard);
    tracing_subscriber::fmt()
        .with_writer(nb)
        .with_ansi(false)
        .try_init()
        .ok();
    Ok(())
}

fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = ratatui::try_restore();
        original(info);
    }));
}
